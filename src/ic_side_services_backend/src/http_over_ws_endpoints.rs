use candid::{CandidType, Deserialize};
use http_over_ws::{HttpMethod, HttpHeader, HttpRequestId, HTTP_REQUESTS, HttpRequestFailureReason, CONNECTED_CLIENTS, ConnectedClients};
use ic_cdk::{
    query, update,
};
use ic_websocket_cdk::*;

use crate::ws::close_client_connection;

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
            state.borrow().get_connected_clients();
        clients
    });

    for client_principal in clients {
        disconnect_client(client_principal);
    }
}