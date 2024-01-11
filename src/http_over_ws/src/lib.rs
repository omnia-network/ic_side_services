use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    future::Future,
    pin::Pin,
    time::Duration,
};

use candid::{decode_one, encode_one, CandidType, Deserialize, Principal};
use ic_cdk::{
    api::management_canister::http_request::{
        HttpHeader as ApiHttpHeader, HttpResponse as ApiHttpResponse,
    },
    trap,
};
use ic_cdk_timers::TimerId;
use logger::log;
use url::Url;

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

impl HttpRequest {
    pub fn new(
        url: &str,
        method: HttpMethod,
        headers: Vec<HttpHeader>,
        body: Option<Vec<u8>>,
    ) -> Self {
        HttpRequest {
            url: url.to_string(),
            method,
            headers,
            body,
        }
    }
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

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        decode_one(bytes).map_err(|e| e.to_string())
    }
}

#[derive(CandidType, Clone, Deserialize)]
pub enum HttpRequestFailureReason {
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
pub struct ConnectedClients(HashMap<Principal, HashSet<HttpRequestId>>);

impl ConnectedClients {
    pub fn new() -> Self {
        ConnectedClients(HashMap::new())
    }

    pub fn add_client(&mut self, client_principal: Principal) {
        self.0.insert(client_principal, HashSet::new());
    }

    pub fn get_client_for_request(&self, request_id: HttpRequestId) -> Option<Principal> {
        let connected_clients_count = self.0.len();
        if connected_clients_count == 0 {
            return None;
        }
        let chosen_client_index = request_id as usize % connected_clients_count;
        // chosen_client_index is in [0, connected_clients_count)
        // where connected_clients_count is the number of clients currently connected.
        // as no client is removed while executing this method,
        // the entry at 'chosen_client_index' is guaranteed to exist
        Some(
            self.0
                .iter()
                .nth(chosen_client_index)
                .expect("client must be connected")
                .0
                .clone(),
        )
    }

    pub fn get_connected_clients(&self) -> Vec<Principal> {
        self.0.keys().cloned().collect()
    }

    pub fn assign_request_to_client(
        &mut self,
        client_principal: &Principal,
        request_id: HttpRequestId,
    ) {
        self.0
            .get_mut(&client_principal)
            .expect("client must be connected")
            .insert(request_id);
    }

    pub fn assign_request(&mut self, request_id: HttpRequestId) -> Result<Principal, String> {
        // pick an arbitrary client
        // TODO: check whether keys are returned in arbitrary order
        let client_principal = self
            .get_client_for_request(request_id)
            .ok_or(String::from("no clients connected"))?
            .clone();
        self.assign_request_to_client(&client_principal, request_id);
        Ok(client_principal)
    }

    pub fn is_request_assigned_to_client(
        &self,
        client_principal: Principal,
        request_id: HttpRequestId,
    ) -> bool {
        self.0
            .get(&client_principal)
            .map(|set| set.contains(&request_id))
            .unwrap_or(false)
    }

    pub fn complete_request_for_client(
        &mut self,
        client_principal: Principal,
        request_id: HttpRequestId,
    ) -> Result<(), String> {
        let client = self.0.get_mut(&client_principal).ok_or(String::from(
            "only requests assigned to connected client can be completed",
        ))?;
        if !client.remove(&request_id) {
            return Err(String::from("client has not been assigned the request"));
        }
        Ok(())
    }

    pub fn remove_client(&mut self, client_principal: &Principal) -> Result<(), String> {
        self.0
            .remove(client_principal)
            .ok_or(String::from("client not connected"))?;
        Ok(())
    }
}

thread_local! {
    /* flexible */ static HTTP_REQUESTS: RefCell<BTreeMap<HttpRequestId, HttpRequestState>> = RefCell::new(BTreeMap::new());
    /* flexible */ static CONNECTED_CLIENTS: RefCell<ConnectedClients> = RefCell::new(ConnectedClients::new());
}

pub fn on_open(client_principal: Principal) {
    // assuming only http proxy connects as a client
    // TODO: handle case in which it's either a ws proxy or a real ws client connecting <
    CONNECTED_CLIENTS.with(|clients| {
        clients.borrow_mut().add_client(client_principal);
    });
    log(&format!(
        "http_over_ws: Client {} connected",
        client_principal
    ))
}

pub fn try_handle_http_over_ws_message(
    client_principal: Principal,
    serialized_message: Vec<u8>,
    ws_send: fn(Principal, Vec<u8>) -> Result<(), String>,
) -> Result<(), String> {
    let incoming_msg = HttpOverWsMessage::from_bytes(&serialized_message)?;

    log(&format!(
        "http_over_ws: incoming message: {:?} from {}",
        incoming_msg, client_principal
    ));

    match incoming_msg {
        HttpOverWsMessage::HttpRequest(_, _) => {
            ws_send(
                client_principal,
                HttpOverWsMessage::Error(
                    None,
                    String::from("Clients are not allowed to send HTTP requests"),
                )
                .to_bytes(),
            )
            .unwrap();
        }
        HttpOverWsMessage::HttpResponse(request_id, response) => {
            if let Err(e) = handle_http_response(client_principal, request_id, response) {
                log(&e);
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
    Ok(())
}

pub fn on_close(client_principal: Principal) {
    CONNECTED_CLIENTS.with(|clients| {
        if let Err(e) = clients.borrow_mut().remove_client(&client_principal) {
            log(&e);
        };
    });

    log(&format!(
        "http_over_ws: Client {} disconnected",
        client_principal
    ))
}

pub fn get_connected_clients() -> ConnectedClients {
    CONNECTED_CLIENTS.with(|clients| clients.borrow().clone())
}

pub fn get_connected_client_principals() -> Vec<Principal> {
    get_connected_clients()
        .0
        .keys()
        .cloned()
        .collect::<Vec<_>>()
}

fn handle_http_response(
    client_principal: Principal,
    request_id: HttpRequestId,
    response: HttpResponse,
) -> Result<(), String> {
    if CONNECTED_CLIENTS.with(|clients| {
        clients
            .borrow()
            .is_request_assigned_to_client(client_principal, request_id)
    }) {
        // assign response to a previous request
        HTTP_REQUESTS.with(|http_requests| -> Result<(), String> {
            let mut h = http_requests.borrow_mut();
            let r = h
                .get_mut(&request_id)
                .ok_or(String::from("request not found"))?;
            r.response = Some(response.clone());

            // response has been received, clear the timer
            let timer_id = r.timer_id.take().ok_or(String::from("timer not set"))?;
            ic_cdk_timers::clear_timer(timer_id);

            let callback = r.callback.ok_or(String::from("callback not set"))?;
            ic_cdk::spawn(async move { callback(response).await });

            Ok(())
        })?;

        CONNECTED_CLIENTS.with(|clients| {
            clients
                .borrow_mut()
                .complete_request_for_client(client_principal, request_id)
        })?;

        log(&format!(
            "http_over_ws: Completed HTTP request {}",
            request_id
        ));

        Ok(())
    } else {
        Err(String::from("request not assigned to client"))
    }
}

fn http_request_timeout(client_principal: Principal, request_id: HttpRequestId) {
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

    if let Err(e) = CONNECTED_CLIENTS.with(|clients| {
        clients
            .borrow_mut()
            .complete_request_for_client(client_principal, request_id)
    }) {
        log(&e);
    }
}

pub fn execute_http_request(
    url: Url,
    method: HttpMethod,
    headers: Vec<HttpHeader>,
    body: Option<String>,
    callback: Option<HttpCallback>,
    timeout_ms: Option<u64>,
    ws_send: fn(Principal, Vec<u8>) -> Result<(), String>,
) -> HttpRequestId {
    let http_request = HttpRequest {
        url: url.to_string(),
        method,
        headers,
        body: body.map(|b| b.into_bytes()),
    };

    let request_id = HTTP_REQUESTS.with(|http_requests| http_requests.borrow().len() + 1) as u32;

    match CONNECTED_CLIENTS.with(|clients| clients.borrow_mut().assign_request(request_id)) {
        Ok(assigned_client_principal) => {
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

            ws_send(
                assigned_client_principal,
                HttpOverWsMessage::HttpRequest(request_id, http_request).to_bytes(),
            )
            .unwrap();
        }
        Err(e) => {
            trap(&e);
        }
    }

    request_id
}

#[derive(CandidType, Deserialize)]
pub struct PrettyHttpRequest {
    url: String,
    method: HttpMethod,
    headers: Vec<HttpHeader>,
    body: Option<String>,
}

pub fn get_http_request(request_id: HttpRequestId) -> Option<PrettyHttpRequest> {
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
pub struct PrettyHttpResponse {
    status: candid::Nat,
    headers: Vec<HttpHeader>,
    body: String,
}

pub type GetHttpResponseResult = Result<PrettyHttpResponse, HttpRequestFailureReason>;

pub fn get_http_response(request_id: HttpRequestId) -> GetHttpResponseResult {
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
        assert!(clients
            .0
            .get(&client_principal)
            .expect("client must be connected")
            .contains(&request_id));

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
        assert!(clients
            .complete_request_for_client(client_principal, request_id)
            .is_ok());
        assert_eq!(
            clients.is_request_assigned_to_client(client_principal, request_id),
            false
        );
    }

    #[test]
    fn should_distribute_requests_among_clients() {
        let mut clients = ConnectedClients::new();

        let client_principal = Principal::from_text("aaaaa-aa").unwrap();
        let another_client_principal = Principal::from_text("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();

        clients.add_client(client_principal);
        clients.add_client(another_client_principal);

        let request_id = 1;
        assert!(clients.assign_request(request_id).is_ok());

        let request_id = 2;
        assert!(clients.assign_request(request_id).is_ok());

        let request_id = 3;
        assert!(clients.assign_request(request_id).is_ok());

        let request_id = 4;
        assert!(clients.assign_request(request_id).is_ok());

        assert!(
            clients
                .0
                .get(&client_principal)
                .expect("client must be connected")
                .len()
                == 2
        );
        assert!(
            clients
                .0
                .get(&another_client_principal)
                .expect("client must be connected")
                .len()
                == 2
        );
    }
}
