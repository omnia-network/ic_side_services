use ic_cdk::{query, update};
use ic_websocket_cdk::*;

use crate::{
    http_over_ws::{on_close, on_message, on_open, HttpOverWsMessage},
    logger::log,
};

pub fn init_ws() {
    let params = WsInitParams::new(WsHandlers {
        on_open: Some(on_open),
        on_message: Some(on_message),
        on_close: Some(on_close),
    });

    ic_websocket_cdk::init(params);
}

pub fn send_ws_message(client_principal: ClientPrincipal, message: HttpOverWsMessage) {
    if let Err(send_err) = ic_websocket_cdk::send(client_principal, message.to_bytes()) {
        log(&format!("ws: Failed to send message: {}", send_err))
    }
}

pub fn close_client_connection(client_principal: ClientPrincipal) {
    if let Err(close_err) = ic_websocket_cdk::close(client_principal) {
        log(&format!("ws: Failed to close connection: {}", close_err))
    }
}

#[update]
fn ws_open(args: CanisterWsOpenArguments) -> CanisterWsOpenResult {
    ic_websocket_cdk::ws_open(args)
}

#[update]
fn ws_close(args: CanisterWsCloseArguments) -> CanisterWsCloseResult {
    ic_websocket_cdk::ws_close(args)
}

#[update]
fn ws_message(
    args: CanisterWsMessageArguments,
    _msg_type: Option<HttpOverWsMessage>,
) -> CanisterWsMessageResult {
    ic_websocket_cdk::ws_message(args, _msg_type)
}

#[query]
fn ws_get_messages(args: CanisterWsGetMessagesArguments) -> CanisterWsGetMessagesResult {
    ic_websocket_cdk::ws_get_messages(args)
}
