use std::{future::Future, pin::Pin};
use http_over_ws::{HttpMethod, HttpResponse, HttpHeader, HttpOverWsError};
use ic_cdk::print;
use ic_cdk_macros::update;
use ic_websocket_cdk::{OnCloseCallbackArgs, OnMessageCallbackArgs, OnOpenCallbackArgs};
use logger::log;
use url::Url;

pub fn on_open(args: OnOpenCallbackArgs) {
    print(format!("WS client: {:?} connected", args.client_principal));
}

pub fn on_message(args: OnMessageCallbackArgs) {
    if let Err(HttpOverWsError::NotHttpOverWsType(_)) = http_over_ws::try_handle_http_over_ws_message(args.client_principal, args.message) {
        log("Received WS client message")
    }
}

pub fn on_close(args: OnCloseCallbackArgs) {
    if let Err(_) = http_over_ws::try_disconnect_http_proxy(args.client_principal) {
        print(format!("WS client {:?} disconnected", args.client_principal));
    } else {
        print(format!("Proxy client {:?} disconnected", args.client_principal));
    }
}

#[update]
fn execute_http_request(url: String, method: HttpMethod, headers: Vec<HttpHeader>, body: Option<String>) -> Result<u32, HttpOverWsError> {
    http_over_ws::execute_http_request(
        Url::parse(&url).unwrap(),
        method,
        headers,
        body,
        Some(callback),
        Some(10_000),
        ic_websocket_cdk::send
    )
}

fn callback(response: HttpResponse) -> Pin<Box<dyn Future<Output = ()>>> {
    Box::pin(async move {
        print(format!("Received response: {:?}", response));
    })
}