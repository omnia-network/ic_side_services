use http_over_ws::{
    ExecuteHttpRequestResult, GetHttpResponseResult, HttpOverWsError, HttpRequest, HttpRequestId,
    HttpResponse,
};
use ic_cdk_macros::{query, update};
use ic_websocket_cdk::{OnCloseCallbackArgs, OnMessageCallbackArgs, OnOpenCallbackArgs};
use logger::log;

pub fn on_open(args: OnOpenCallbackArgs) {
    log(&format!("WS client: {:?} connected", args.client_principal));
}

pub fn on_message(args: OnMessageCallbackArgs) {
    if let Err(HttpOverWsError::NotHttpOverWsType(_)) =
        http_over_ws::try_handle_http_over_ws_message(args.client_principal, args.message.clone())
    {
        log(&format!(
            "Received WS client message: {:?} from {}",
            args.message, args.client_principal
        ));
    }
}

pub fn on_close(args: OnCloseCallbackArgs) {
    if let Err(_) = http_over_ws::try_disconnect_http_proxy(args.client_principal) {
        log(&format!(
            "WS client {:?} disconnected",
            args.client_principal
        ));
    } else {
        log(&format!(
            "Proxy client {:?} disconnected",
            args.client_principal
        ));
    }
}

#[update]
fn execute_http_request(req: HttpRequest) -> ExecuteHttpRequestResult {
    http_over_ws::execute_http_request(
        req,
        Some(|res| Box::pin(callback(res))),
        Some(10_000),
        ic_websocket_cdk::send,
    )
}

async fn callback(response: HttpResponse) {
    log(&format!("Response: {:?}", response));
}

#[query]
fn get_http_response(id: HttpRequestId) -> GetHttpResponseResult {
    http_over_ws::get_http_response(id)
}