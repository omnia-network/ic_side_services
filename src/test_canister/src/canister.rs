use http_over_ws::{
    HttpOverWsError, HttpRequest, HttpRequestFailureReason, HttpRequestId, HttpResponse,
    PrettyHttpResponse,
};
use ic_cdk_macros::{query, update};
use ic_websocket_cdk::{OnCloseCallbackArgs, OnMessageCallbackArgs, OnOpenCallbackArgs};
use logger::log;
use url::Url;

pub fn on_open(args: OnOpenCallbackArgs) {
    log(&format!("Client: {:?} connected", args.client_principal));
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
    http_over_ws::on_close(args.client_principal);
}

#[update]
fn execute_http_request(req: HttpRequest) -> u32 {
    http_over_ws::execute_http_request(
        Url::parse(&req.url).unwrap(),
        req.method,
        req.headers,
        req.body.map(|b| String::from_utf8_lossy(&b).to_string()), // TODO: change execute_http_request to take Vec<u8>
        Some(|res| Box::pin(callback(res))),
        Some(10_000),
        ic_websocket_cdk::send,
    )
}

async fn callback(response: HttpResponse) {
    log(&format!("Response: {:?}", response));
}

#[query]
fn get_http_response(id: HttpRequestId) -> Result<PrettyHttpResponse, HttpRequestFailureReason> {
    http_over_ws::get_http_response(id)
}
