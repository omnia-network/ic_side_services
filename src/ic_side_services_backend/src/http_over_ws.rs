use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
};

use candid::{decode_one, encode_one, CandidType, Deserialize};
use ic_cdk::{
    api::management_canister::http_request::{HttpHeader, HttpResponse},
    query, trap, update,
};
use ic_websocket_cdk::*;

use crate::{logger::log, ws::send_ws_message};

type HttpRequestId = u32;

#[derive(CandidType, Clone, Deserialize)]
enum HttpMethod {
    GET,
    POST,
    PUT,
    HEAD,
    DELETE,
}

#[derive(CandidType, Clone, Deserialize)]
pub struct HttpRequest {
    url: String,
    method: HttpMethod,
    headers: Vec<HttpHeader>,
    body: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize)]
pub enum HttpOverWsMessage {
    HttpRequest(HttpRequestId, HttpRequest),
    HttpResponse(HttpRequestId, HttpResponse),
    Error(String),
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
struct HttpRequestState {
    request: HttpRequest,
    response: Option<HttpResponse>,
}

impl HttpRequestState {
    fn new(request: HttpRequest) -> Self {
        HttpRequestState {
            request,
            response: None,
        }
    }
}

#[derive(CandidType, Clone, Deserialize)]
struct ConnectedClients {
    idle_clients: HashSet<ClientPrincipal>,
    busy_clients: HashMap<ClientPrincipal, HashSet<HttpRequestId>>,
}

impl ConnectedClients {
    fn new() -> Self {
        ConnectedClients {
            idle_clients: HashSet::new(),
            busy_clients: HashMap::new(),
        }
    }

    fn add_client(&mut self, client_principal: ClientPrincipal) {
        self.idle_clients.insert(client_principal);
    }

    fn assign_request_to_client(
        &mut self,
        client_principal: ClientPrincipal,
        request_id: HttpRequestId,
    ) {
        self.idle_clients.remove(&client_principal);
        self.busy_clients
            .entry(client_principal)
            .or_default()
            .insert(request_id);
    }

    fn assign_request(&mut self, request_id: HttpRequestId) -> Option<ClientPrincipal> {
        if let Some(client_principal) = self.idle_clients.iter().next().cloned() {
            self.assign_request_to_client(client_principal, request_id);
            Some(client_principal)
        } else {
            // pick an arbitrary busy client
            if let Some(client_principal) = self.busy_clients.keys().next().cloned() {
                self.assign_request_to_client(client_principal, request_id);
                return Some(client_principal);
            }
            None
        }
    }

    fn is_request_assigned_to_client(
        &self,
        client_principal: ClientPrincipal,
        request_id: HttpRequestId,
    ) -> bool {
        self.busy_clients
            .get(&client_principal)
            .map(|set| set.contains(&request_id))
            .unwrap_or(false)
    }

    fn complete_request_for_client(
        &mut self,
        client_principal: ClientPrincipal,
        request_id: HttpRequestId,
    ) {
        let busy_client = self.busy_clients.get_mut(&client_principal).unwrap();
        busy_client.remove(&request_id);

        if busy_client.is_empty() {
            self.busy_clients.remove(&client_principal);
            self.idle_clients.insert(client_principal);
        }
    }

    fn remove_client(&mut self, client_principal: &ClientPrincipal) {
        self.idle_clients.remove(client_principal);
        self.busy_clients.remove(client_principal);
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

    match incoming_msg {
        HttpOverWsMessage::HttpRequest(_, _) => {
            send_ws_message(
                client_principal,
                HttpOverWsMessage::Error(String::from(
                    "Clients are not allowed to send HTTP requests",
                )),
            );
        }
        HttpOverWsMessage::HttpResponse(request_id, response) => {
            if CONNECTED_CLIENTS.with(|clients| {
                clients
                    .borrow_mut()
                    .is_request_assigned_to_client(client_principal, request_id)
            }) {
                HTTP_REQUESTS.with(|http_requests| {
                    http_requests
                        .borrow_mut()
                        .get_mut(&request_id)
                        .and_then(|request| {
                            request.response = Some(response);
                            Some(request)
                        });
                });

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
        HttpOverWsMessage::Error(err) => {
            log(&format!("http_over_ws: incoming error: {}", err));
        }
    };
}

pub fn on_close(args: OnCloseCallbackArgs) {
    CONNECTED_CLIENTS.with(|clients| {
        clients.borrow_mut().remove_client(&args.client_principal);
    });
}

#[update]
fn execute_http_request(
    url: String,
    method: HttpMethod,
    headers: Vec<HttpHeader>,
    body: Option<String>,
) -> HttpRequestId {
    let http_request = HttpRequest {
        url,
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

    if let Some(assigned_client_principal) =
        CONNECTED_CLIENTS.with(|clients| clients.borrow_mut().assign_request(request_id))
    {
        HTTP_REQUESTS.with(|http_requests| {
            http_requests
                .borrow_mut()
                .insert(request_id, HttpRequestState::new(http_request.clone()));
        });

        send_ws_message(
            assigned_client_principal,
            HttpOverWsMessage::HttpRequest(request_id, http_request),
        );
    } else {
        trap("No available clients");
    }

    request_id
}

#[derive(CandidType, Deserialize)]
struct PrettyHttpResponse {
    status: candid::Nat,
    headers: Vec<HttpHeader>,
    body: String,
}

#[query]
fn get_http_response(request_id: HttpRequestId) -> Option<PrettyHttpResponse> {
    HTTP_REQUESTS.with(|http_requests| {
        http_requests
            .borrow()
            .get(&request_id)
            .map(|r| {
                r.response.as_ref().map(|res| PrettyHttpResponse {
                    status: res.status.clone(),
                    headers: res.headers.clone(),
                    body: String::from_utf8_lossy(&res.body).to_string(),
                })
            })
            .flatten()
    })
}

#[query]
fn get_connected_clients() -> ConnectedClients {
    CONNECTED_CLIENTS.with(|clients| clients.borrow().clone())
}
