use http_over_ws::{HttpRequestId, ConnectedClients, PrettyHttpRequest, GetHttpResponseResult};
use ic_cdk::{
    query, update,
};
use ic_websocket_cdk::ClientPrincipal;
use logger::log;

#[query]
fn get_http_request(request_id: HttpRequestId) -> Option<PrettyHttpRequest> {
    http_over_ws::get_http_request(request_id)
}

#[query]
fn get_http_response(request_id: HttpRequestId) -> GetHttpResponseResult {
    http_over_ws::get_http_response(request_id)
}

#[query]
fn get_connected_clients() -> ConnectedClients {
    http_over_ws::get_connected_clients()
}

#[update]
fn disconnect_client(client_principal: ClientPrincipal) {
    if let Err(close_err) = ic_websocket_cdk::close(client_principal) {
        log(&format!("ws: Failed to close connection: {}", close_err))
    }
}

#[update]
pub fn disconnect_all_clients() {
    let clients = http_over_ws::get_connected_client_principals();

    for client_principal in clients {
        disconnect_client(client_principal);
    }
}