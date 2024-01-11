use http_over_ws::{GetHttpResponseResult, HttpRequest, HttpRequestId};
use ic_cdk::{query, update};
use ic_websocket_cdk::ClientPrincipal;
use logger::log;

#[query]
fn get_http_request(request_id: HttpRequestId) -> Option<HttpRequest> {
    http_over_ws::get_http_request(request_id)
}

#[query]
fn get_http_response(request_id: HttpRequestId) -> GetHttpResponseResult {
    http_over_ws::get_http_response(request_id)
}

#[update]
fn disconnect_client(client_principal: ClientPrincipal) {
    if let Err(close_err) = ic_websocket_cdk::close(client_principal) {
        log(&format!("ws: Failed to close connection: {}", close_err))
    }
}
