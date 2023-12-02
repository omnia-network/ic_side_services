use ic_cdk::{query, update};
use ic_websocket_cdk::*;

use crate::{
    http_over_ws::{on_close, on_message, on_open, HttpOverWsMessage},
    logger::log,
};

pub const GATEWAY_PRINCIPAL: &str =
    "3656s-3kqlj-dkm5d-oputg-ymybu-4gnuq-7aojd-w2fzw-5lfp2-4zhx3-4ae";

pub fn init_ws() {
    let params = WsInitParams::new(
        WsHandlers {
            on_open: Some(on_open),
            on_message: Some(on_message),
            on_close: Some(on_close),
        },
        vec![String::from(GATEWAY_PRINCIPAL)],
    );

    ic_websocket_cdk::init(params);
}

pub fn send_ws_message(client_principal: ClientPrincipal, message: HttpOverWsMessage) {
    if let Err(send_err) = ic_websocket_cdk::ws_send(client_principal, message.to_bytes()) {
        log(&format!("ws: Failed to send message: {}", send_err))
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
