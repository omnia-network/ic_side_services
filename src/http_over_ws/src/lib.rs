use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    future::Future,
    pin::Pin,
    time::Duration,
};

use candid::{decode_one, encode_one, CandidType, Deserialize, Principal};
use ic_cdk::api::management_canister::http_request::{
    HttpHeader as ApiHttpHeader, HttpResponse as ApiHttpResponse,
};
use ic_cdk_timers::TimerId;
use logger::log;

pub type HttpRequestId = u32;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    HEAD,
    DELETE,
}

pub type HttpHeader = ApiHttpHeader;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Vec<HttpHeader>,
    pub body: Option<Vec<u8>>,
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

#[derive(CandidType, Debug, Deserialize, PartialEq, Eq)]
pub enum HttpOverWsMessage {
    SetupProxyClient,
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

#[derive(CandidType, Debug, Deserialize)]
pub enum HttpOverWsError {
    /// The message is not an HttpOverWsMessage, therefore the on_message callback given to the IC WS cdk
    /// should try to parse it as its own message type.
    NotHttpOverWsType(String),
    /// The message is an HttpOverWsMessage, however it is not what it is expected to be.
    InvalidHttpMessage(HttpFailureReason),
}

#[derive(CandidType, Clone, Debug, Deserialize)]
pub enum HttpFailureReason {
    RequestTimeout,
    ProxyError(String),
    /// Used when retrieving the request from the state
    /// and the request is not found.
    RequestIdNotFound,
    TimerNotSet,
    CallbackNotSet,
    Unknown,
}

#[derive(Clone)]
struct HttpRequestState {
    request: HttpRequest,
    response: Option<HttpResponse>,
    callback: Option<HttpCallback>,
    timer_id: Option<TimerId>,
    failure_reason: Option<HttpFailureReason>,
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

    fn add_client(&mut self, client_principal: Principal) {
        self.0.insert(client_principal, HashSet::new());
    }

    fn get_client_for_request(&self, request_id: HttpRequestId) -> Option<Principal> {
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
                .expect("client is not connected")
                .0
                .clone(),
        )
    }

    fn assign_request_to_client(
        &mut self,
        client_principal: &Principal,
        request_id: HttpRequestId,
    ) -> Result<(), HttpFailureReason> {
        self.0
            .get_mut(&client_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "proxy not connected",
            )))?
            .insert(request_id);
        Ok(())
    }

    fn assign_request(
        &mut self,
        request_id: HttpRequestId,
    ) -> Result<Principal, HttpFailureReason> {
        let client_principal = self
            .get_client_for_request(request_id)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "no clients connected",
            )))?
            .clone();
        self.assign_request_to_client(&client_principal, request_id)?;
        Ok(client_principal)
    }

    fn complete_request_for_client(
        &mut self,
        client_principal: Principal,
        request_id: HttpRequestId,
    ) -> Result<(), HttpFailureReason> {
        let client = self
            .0
            .get_mut(&client_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "proxy not connected",
            )))?;
        client
            .remove(&request_id)
            .then(|| ())
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "client has not been assigned the request",
            )))?;
        Ok(())
    }

    fn remove_client(&mut self, client_principal: &Principal) -> Result<(), HttpFailureReason> {
        self.0
            .remove(client_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "client not connected",
            )))?;
        Ok(())
    }
}

thread_local! {
    /* flexible */ static HTTP_REQUESTS: RefCell<BTreeMap<HttpRequestId, HttpRequestState>> = RefCell::new(BTreeMap::new());
    /* flexible */ static CONNECTED_CLIENTS: RefCell<ConnectedClients> = RefCell::new(ConnectedClients::new());
}

/// Called by the callbacl passed to the IC WS cdk when a new message is received.
/// Checks if the message is an HttpOverWsMessage, and if so it handles it.
/// Otherwise, it returns 'HttpOverWsError::NotHttpOverWsType' to signal the callback that it should treat it as a WS message sent by one of the WS clients (and not an HTTP Proxy)
pub fn try_handle_http_over_ws_message(
    client_principal: Principal,
    serialized_message: Vec<u8>,
) -> Result<(), HttpOverWsError> {
    let incoming_msg = HttpOverWsMessage::from_bytes(&serialized_message)
        .map_err(|e| HttpOverWsError::NotHttpOverWsType(e))?;

    log(&format!(
        "http_over_ws: incoming message: {:?} from {}",
        incoming_msg, client_principal
    ));

    match incoming_msg {
        HttpOverWsMessage::SetupProxyClient => {
            CONNECTED_CLIENTS.with(|clients| {
                clients.borrow_mut().add_client(client_principal);
            });
            log(&format!(
                "http_over_ws: proxy client {} connected",
                client_principal
            ));
            Ok(())
        }
        HttpOverWsMessage::HttpResponse(request_id, response) => {
            handle_http_response(client_principal, request_id, response)?;
            Ok(())
        }
        HttpOverWsMessage::Error(request_id, err) => {
            let e = format!("http_over_ws: incoming error: {}", err);
            log(&err);

            if let Some(request_id) = request_id {
                HTTP_REQUESTS.with(|http_requests| {
                    http_requests
                        .borrow_mut()
                        .get_mut(&request_id)
                        .and_then(|r| {
                            r.failure_reason = Some(HttpFailureReason::ProxyError(err));
                            Some(r)
                        });
                });
            }
            Err(HttpOverWsError::InvalidHttpMessage(
                HttpFailureReason::ProxyError(e),
            ))
        }
        HttpOverWsMessage::HttpRequest(_, _) => {
            let e = String::from(
                "http_over_ws: proxy client is not allowed to send HTTP requests over WS",
            );
            log(&e);
            Err(HttpOverWsError::InvalidHttpMessage(
                HttpFailureReason::ProxyError(e),
            ))
        }
    }
}

pub fn try_disconnect_http_proxy(client_principal: Principal) -> Result<(), HttpFailureReason> {
    CONNECTED_CLIENTS.with(|clients| clients.borrow_mut().remove_client(&client_principal))?;

    log(&format!(
        "http_over_ws: Client {} disconnected",
        client_principal
    ));
    Ok(())
}

fn handle_http_response(
    client_principal: Principal,
    request_id: HttpRequestId,
    response: HttpResponse,
) -> Result<(), HttpOverWsError> {
    CONNECTED_CLIENTS.with(|clients| {
        clients
            .borrow_mut()
            .complete_request_for_client(client_principal, request_id)
            .map_err(|e| HttpOverWsError::InvalidHttpMessage(e))
    })?;

    // assign response to a previous request
    HTTP_REQUESTS.with(|http_requests| -> Result<(), HttpOverWsError> {
        let mut h = http_requests.borrow_mut();
        let r = h
            .get_mut(&request_id)
            .ok_or(HttpOverWsError::InvalidHttpMessage(
                HttpFailureReason::RequestIdNotFound,
            ))?;
        r.response = Some(response.clone());

        // response has been received, clear the timer
        let timer_id = r
            .timer_id
            .take()
            .ok_or(HttpOverWsError::InvalidHttpMessage(
                HttpFailureReason::TimerNotSet,
            ))?;
        ic_cdk_timers::clear_timer(timer_id);

        let callback = r.callback.ok_or(HttpOverWsError::InvalidHttpMessage(
            HttpFailureReason::CallbackNotSet,
        ))?;
        ic_cdk::spawn(async move { callback(response).await });

        Ok(())
    })?;

    log(&format!(
        "http_over_ws: completed HTTP request {}",
        request_id
    ));

    Ok(())
}

fn http_request_timeout(client_principal: Principal, request_id: HttpRequestId) {
    if let Err(_) = CONNECTED_CLIENTS.with(|clients| {
        clients
            .borrow_mut()
            .complete_request_for_client(client_principal, request_id)
    }) {
        log("cannot complete requst");
    }

    HTTP_REQUESTS.with(|http_requests| {
        http_requests
            .borrow_mut()
            .get_mut(&request_id)
            .and_then(|r| {
                if r.response.is_none() {
                    r.failure_reason = Some(HttpFailureReason::RequestTimeout);

                    log(&format!(
                        "http_over_ws: HTTP request with id {} timed out",
                        request_id
                    ));
                }

                Some(r)
            });
    });
}

pub type ExecuteHttpRequestResult = Result<HttpRequestId, HttpOverWsError>;

pub fn execute_http_request(
    req: HttpRequest,
    callback: Option<HttpCallback>,
    timeout_ms: Option<u64>,
    ws_send: fn(Principal, Vec<u8>) -> Result<(), String>,
) -> ExecuteHttpRequestResult {
    let request_id = HTTP_REQUESTS.with(|http_requests| http_requests.borrow().len() + 1) as u32;

    let assigned_client_principal = CONNECTED_CLIENTS
        .with(|clients| clients.borrow_mut().assign_request(request_id))
        .map_err(|e| HttpOverWsError::InvalidHttpMessage(e))?;

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
            HttpRequestState::new(req.clone(), callback, timer_id),
        );
    });

    ws_send(
        assigned_client_principal,
        HttpOverWsMessage::HttpRequest(request_id, req).to_bytes(),
    )
    .unwrap();

    Ok(request_id)
}

pub fn get_http_request(request_id: HttpRequestId) -> Option<HttpRequest> {
    HTTP_REQUESTS.with(|http_requests| {
        http_requests
            .borrow()
            .get(&request_id)
            .map(|r| r.request.to_owned())
    })
}

pub type GetHttpResponseResult = Result<HttpResponse, HttpFailureReason>;

pub fn get_http_response(request_id: HttpRequestId) -> GetHttpResponseResult {
    HTTP_REQUESTS.with(|http_requests| {
        http_requests
            .borrow()
            .get(&request_id)
            .ok_or(HttpFailureReason::RequestIdNotFound)
            .map(|r| {
                r.response
                    .as_ref()
                    .ok_or(
                        r.failure_reason
                            .clone()
                            .unwrap_or(HttpFailureReason::Unknown),
                    )
                    .cloned()
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
            .expect("client is not connected")
            .contains(&request_id));
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
        assert!(clients
            .complete_request_for_client(client_principal, request_id)
            .is_ok());
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
                .expect("client is not connected")
                .len()
                == 2
        );
        assert!(
            clients
                .0
                .get(&another_client_principal)
                .expect("client is not connected")
                .len()
                == 2
        );
    }
}
