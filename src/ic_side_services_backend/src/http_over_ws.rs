use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    future::Future,
    pin::Pin,
    time::Duration,
};

use candid::{decode_one, encode_one, CandidType, Deserialize};
use ic_cdk::{
    api::management_canister::http_request::{
        HttpHeader as ApiHttpHeader, HttpResponse as ApiHttpResponse,
    },
    print, query, trap, update,
};
use ic_cdk_timers::TimerId;
use ic_websocket_cdk::*;
use url::Url;

use crate::{
    logger::log,
    ws::{close_client_connection, send_ws_message},
};

pub type HttpRequestId = u32;

#[derive(CandidType, Clone, Debug, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    HEAD,
    DELETE,
}

pub type HttpHeader = ApiHttpHeader;

#[derive(CandidType, Clone, Debug, Deserialize)]
pub struct HttpRequest {
    url: String,
    method: HttpMethod,
    headers: Vec<HttpHeader>,
    body: Option<Vec<u8>>,
}

pub type HttpResponse = ApiHttpResponse;
pub type HttpCallback = fn(HttpResponse) -> Pin<Box<dyn Future<Output = ()>>>;

#[derive(CandidType, Debug, Deserialize)]
pub enum HttpOverWsMessage {
    HttpRequest(HttpRequestId, HttpRequest),
    HttpResponse(HttpRequestId, HttpResponse),
    Error(Option<HttpRequestId>, String),
}

impl HttpOverWsMessage {
    pub fn to_bytes(&self) -> Vec<u8> {
        encode_one(self).unwrap()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        decode_one(bytes).unwrap()
    }
}

#[derive(CandidType, Clone, Deserialize)]
enum HttpRequestFailureReason {
    Timeout,
    ErrorFromClient(String),
    /// Used when retrieving the request from the state
    /// and the request is not found.
    NotFound,
    Unknown,
}

#[derive(Clone)]
struct HttpRequestState {
    request: HttpRequest,
    response: Option<HttpResponse>,
    callback: Option<HttpCallback>,
    timer_id: Option<TimerId>,
    failure_reason: Option<HttpRequestFailureReason>,
}

impl HttpRequestState {
    fn new(
        request: HttpRequest,
        callback: Option<HttpCallback>,
        timer_id: Option<TimerId>,
    ) -> Self {
        HttpRequestState {
            request,
            response: None,
            callback,
            timer_id,
            failure_reason: None,
        }
    }
}

#[derive(CandidType, Clone, Deserialize)]
struct ConnectedClients(HashMap<ClientPrincipal, HashSet<HttpRequestId>>);

impl ConnectedClients {
    fn new() -> Self {
        ConnectedClients(HashMap::new())
    }

    fn add_client(&mut self, client_principal: ClientPrincipal) {
        self.0.insert(client_principal, HashSet::new());
    }

    fn assign_request_to_client(
        &mut self,
        client_principal: &ClientPrincipal,
        request_id: HttpRequestId,
    ) {
        self.0
            .get_mut(&client_principal)
            .expect("client must be connected")
            .insert(request_id);
    }

    fn assign_request(&mut self, request_id: HttpRequestId) -> Result<ClientPrincipal, String> {
        // pick an arbitrary client
        // TODO: check whether keys are returned in arbitrary order
        let client_principal = self.0.keys().next()
            .ok_or(String::from("no clients connected"))?.clone();
        self.assign_request_to_client(&client_principal, request_id);
        Ok(client_principal)
    }

    fn is_request_assigned_to_client(
        &self,
        client_principal: ClientPrincipal,
        request_id: HttpRequestId,
    ) -> bool {
        self.0
            .get(&client_principal)
            .map(|set| set.contains(&request_id))
            .unwrap_or(false)
    }

    fn complete_request_for_client(
        &mut self,
        client_principal: ClientPrincipal,
        request_id: HttpRequestId,
    ) -> Result<(), String> {
        let client = self.0.get_mut(&client_principal)
            .ok_or(String::from("only requests assigned to connected client can be completed"))?;
        if !client.remove(&request_id) {
            return Err(String::from("client has not been assigned the request"));
        }
        Ok(())
    }

    fn remove_client(&mut self, client_principal: &ClientPrincipal) -> Result<(), String> {
        self.0.remove(client_principal).ok_or(String::from("client not connected"))?;
        Ok(())
    }
}

thread_local! {
    /* flexible */ static HTTP_REQUESTS: RefCell<BTreeMap<HttpRequestId, HttpRequestState>> = RefCell::new(BTreeMap::new());
    /* flexible */ static CONNECTED_CLIENTS: RefCell<ConnectedClients> = RefCell::new(ConnectedClients::new());
}

pub fn on_open(args: OnOpenCallbackArgs) {
    CONNECTED_CLIENTS.with(|clients| {
        clients.borrow_mut().add_client(args.client_principal);
    });
}

pub fn on_message(args: OnMessageCallbackArgs) {
    let incoming_msg = HttpOverWsMessage::from_bytes(&args.message);
    let client_principal = args.client_principal;

    log(&format!(
        "http_over_ws: incoming message: {:?} from {}",
        incoming_msg, client_principal
    ));

    match incoming_msg {
        HttpOverWsMessage::HttpRequest(_, _) => {
            send_ws_message(
                client_principal,
                HttpOverWsMessage::Error(
                    None,
                    String::from("Clients are not allowed to send HTTP requests"),
                ),
            );
        }
        HttpOverWsMessage::HttpResponse(request_id, response) => {
            if CONNECTED_CLIENTS.with(|clients| {
                clients
                    .borrow_mut()
                    .is_request_assigned_to_client(client_principal, request_id)
            }) {
                match &mut HTTP_REQUESTS
                    .with(|http_requests| http_requests.borrow().get(&request_id).cloned())
                {
                    Some(r) => {
                        r.response = Some(response.clone());

                        // response have been received, clear the timer
                        if let Some(timer_id) = r.timer_id.take() {
                            ic_cdk_timers::clear_timer(timer_id);
                        }

                        HTTP_REQUESTS.with(|http_requests| {
                            http_requests.borrow_mut().insert(request_id, r.clone())
                        });

                        if let Some(callback) = r.callback {
                            ic_cdk::spawn(async move { callback(response).await });
                        }
                    }
                    None => {}
                };

                CONNECTED_CLIENTS.with(|clients| {
                    clients
                        .borrow_mut()
                        .complete_request_for_client(client_principal, request_id);
                });

                log(&format!(
                    "http_over_ws: Completed HTTP request {}",
                    request_id
                ));
            }
        }
        HttpOverWsMessage::Error(request_id, err) => {
            log(&format!("http_over_ws: incoming error: {}", err));

            if let Some(request_id) = request_id {
                HTTP_REQUESTS.with(|http_requests| {
                    http_requests
                        .borrow_mut()
                        .get_mut(&request_id)
                        .and_then(|r| {
                            r.failure_reason = Some(HttpRequestFailureReason::ErrorFromClient(err));

                            Some(r)
                        });
                });
            }
        }
    };
}

pub fn on_close(args: OnCloseCallbackArgs) {
    CONNECTED_CLIENTS.with(|clients| {
        clients.borrow_mut().remove_client(&args.client_principal);
    });

    print(&format!(
        "http_over_ws: Client {} disconnected",
        args.client_principal
    ))
}

fn http_request_timeout(client_principal: ClientPrincipal, request_id: HttpRequestId) {
    HTTP_REQUESTS.with(|http_requests| {
        http_requests
            .borrow_mut()
            .get_mut(&request_id)
            .and_then(|r| {
                if r.response.is_none() {
                    r.failure_reason = Some(HttpRequestFailureReason::Timeout);

                    log(&format!(
                        "http_over_ws: HTTP request with id {} timed out",
                        request_id
                    ));
                }

                Some(r)
            });
    });

    CONNECTED_CLIENTS.with(|clients| {
        clients
            .borrow_mut()
            .complete_request_for_client(client_principal, request_id);
    })
}

pub fn execute_http_request(
    url: Url,
    method: HttpMethod,
    headers: Vec<HttpHeader>,
    body: Option<String>,
    callback: Option<HttpCallback>,
    timeout_ms: Option<u64>,
) -> HttpRequestId {
    let http_request = HttpRequest {
        url: url.to_string(),
        method,
        headers,
        body: body.map(|b| b.into_bytes()),
    };

    let request_id = HTTP_REQUESTS.with(|http_requests| {
        if let Some((r, _)) = http_requests.borrow().last_key_value() {
            r + 1
        } else {
            1
        }
    });

    if let Ok(assigned_client_principal) =
        CONNECTED_CLIENTS.with(|clients| clients.borrow_mut().assign_request(request_id))
    {
        let timer_id = match timeout_ms {
            Some(millis) => Some(ic_cdk_timers::set_timer(
                Duration::from_millis(millis),
                move || {
                    http_request_timeout(assigned_client_principal, request_id);
                },
            )),
            None => None,
        };

        HTTP_REQUESTS.with(|http_requests| {
            http_requests.borrow_mut().insert(
                request_id,
                HttpRequestState::new(http_request.clone(), callback, timer_id),
            );
        });

        send_ws_message(
            assigned_client_principal,
            HttpOverWsMessage::HttpRequest(request_id, http_request),
        );
    } else {
        trap("No available HTTP clients");
    }

    request_id
}

#[derive(CandidType, Deserialize)]
struct PrettyHttpRequest {
    url: String,
    method: HttpMethod,
    headers: Vec<HttpHeader>,
    body: Option<String>,
}

#[query]
fn get_http_request(request_id: HttpRequestId) -> Option<PrettyHttpRequest> {
    HTTP_REQUESTS.with(|http_requests| {
        http_requests
            .borrow()
            .get(&request_id)
            .map(|r| PrettyHttpRequest {
                url: r.request.url.clone(),
                method: r.request.method.clone(),
                headers: r.request.headers.clone(),
                body: r
                    .request
                    .body
                    .as_ref()
                    .map(|b| String::from_utf8_lossy(b).to_string()),
            })
    })
}

#[derive(CandidType, Deserialize)]
struct PrettyHttpResponse {
    status: candid::Nat,
    headers: Vec<HttpHeader>,
    body: String,
}

type GetHttpResponseResult = Result<PrettyHttpResponse, HttpRequestFailureReason>;

#[query]
fn get_http_response(request_id: HttpRequestId) -> GetHttpResponseResult {
    HTTP_REQUESTS.with(|http_requests| {
        http_requests
            .borrow()
            .get(&request_id)
            .ok_or(HttpRequestFailureReason::NotFound)
            .map(|r| {
                r.response
                    .as_ref()
                    .ok_or(
                        r.failure_reason
                            .clone()
                            .unwrap_or(HttpRequestFailureReason::Unknown),
                    )
                    .map(|res| PrettyHttpResponse {
                        status: res.status.clone(),
                        headers: res.headers.clone(),
                        body: String::from_utf8_lossy(&res.body).to_string(),
                    })
            })?
    })
}

#[query]
fn get_connected_clients() -> ConnectedClients {
    CONNECTED_CLIENTS.with(|clients| clients.borrow().clone())
}

#[update]
fn disconnect_client(client_principal: ClientPrincipal) {
    close_client_connection(client_principal);
}

#[update]
pub fn disconnect_all_clients() {
    let clients = CONNECTED_CLIENTS.with(|state| {
        let clients: Vec<ClientPrincipal> =
            state.borrow().0.keys().cloned().collect();
        clients
    });

    for client_principal in clients {
        disconnect_client(client_principal);
    }
}

#[cfg(test)]
mod tests {
    use candid::Principal;

    use super::*;

    #[test]
    fn should_add_client_and_assign_request() {
        let mut clients = ConnectedClients::new();
        let client_principal = Principal::anonymous();
        clients.add_client(client_principal);
        assert_eq!(clients.0.len(), 1);

        let request_id = 1;
        assert!(clients.assign_request(request_id).is_ok());
        assert!(clients.0.get(&client_principal).expect("client must be connected").contains(&request_id));

        assert!(clients.is_request_assigned_to_client(client_principal, request_id));
    }

    #[test]
    fn should_not_assign_request() {
        let mut clients = ConnectedClients::new();
        assert!(clients.assign_request(1).is_err());
    }

    #[test]
    fn should_complete_request() {
        let mut clients = ConnectedClients::new();

        let client_principal = Principal::anonymous();
        clients.add_client(client_principal);
        let request_id = 1;
        assert!(clients.assign_request(request_id).is_ok());
        assert!(clients.is_request_assigned_to_client(client_principal, request_id));
        assert!(clients.complete_request_for_client(client_principal, request_id).is_ok());
        assert_eq!(clients.is_request_assigned_to_client(client_principal, request_id), false);
    }
}