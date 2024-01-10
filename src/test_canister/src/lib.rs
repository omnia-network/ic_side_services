use candid::{Principal, Nat};
use ic_cdk_macros::*;
use ic_websocket_cdk::{OnMessageCallbackArgs, CanisterWsOpenArguments};
use http_over_ws::{HttpMethod, HttpResponse, HttpOverWsMessage, init_ws};
use url::Url;
use logger::Logs;

#[init]
fn init() {
    init_ws();

    ic_websocket_cdk::ws_open(CanisterWsOpenArguments::new(0, Principal::from_text("siv4u-w6soj-sx6qh-skhd4-gktr2-2rylx-v5ble-yc73i-mdbvl-gwe32-iqe").unwrap())).unwrap();
}

#[post_upgrade]
fn post_upgrade() {
    init();
}

#[update]
fn execute_http_request() -> u32 {
    http_over_ws::execute_http_request(
        Url::parse("https://omnia-iot.com").unwrap(),
        HttpMethod::GET, vec![], 
        None, 
        None, 
        None
    )
}

#[update]
fn handle_http_response(request_id: u32) {
    let bytes = HttpOverWsMessage::HttpResponse(request_id, HttpResponse {
        status: Nat::from(200),
        headers: vec![],
        body: vec![],
    }).to_bytes();
    let args = OnMessageCallbackArgs {
        client_principal: Principal::from_text("siv4u-w6soj-sx6qh-skhd4-gktr2-2rylx-v5ble-yc73i-mdbvl-gwe32-iqe").unwrap(),
        message: bytes
    };
    http_over_ws::on_message(args);
}

#[query]
fn get_logs() -> Logs {
    logger::get_logs()
}