use crate::{http_connection::*, state::STATE};
use candid::Principal;
use logger::log;

/// Called by the callback passed to the IC WS cdk when a new message is received.
/// Checks if the message is an [HttpOverWsMessage], and if so it handles it.
/// Otherwise, it returns [`HttpOverWsError::NotHttpOverWsType`] to signal the callback that it should treat it as a WS message sent by one of the WS proxies (and not an HTTP Proxy)
pub fn try_handle_http_over_ws_message(
    proxy_principal: Principal,
    serialized_message: Vec<u8>,
) -> Result<(), HttpOverWsError> {
    let incoming_msg = HttpOverWsMessage::from_bytes(&serialized_message)
        .map_err(|e| HttpOverWsError::NotHttpOverWsType(e))?;

    log(&format!(
        "http_over_ws: incoming message: {:?} from {}",
        incoming_msg, proxy_principal
    ));

    match incoming_msg {
        HttpOverWsMessage::SetupProxyClient => {
            STATE.with(|state| {
                state.borrow_mut().add_proxy(proxy_principal);
            });
            log(&format!(
                "http_over_ws: client proxy {} connected",
                proxy_principal
            ));
        }
        HttpOverWsMessage::HttpResponse(request_id, response) => {
            handle_http_result(proxy_principal, request_id, HttpResult::Success(response));
        }
        HttpOverWsMessage::Error(request_id, err) => {
            if let Some(request_id) = request_id {
                handle_http_result(
                    proxy_principal,
                    request_id,
                    HttpResult::Failure(HttpFailureReason::ProxyError(err)),
                );
            } else {
                log(&format!("http_over_ws: incoming error: {}", err));
            }
        }
        HttpOverWsMessage::HttpRequest(_, _) => {
            let e = String::from(
                "http_over_ws: client proxy is not allowed to send HTTP connections over WS",
            );
            log(&e);
        }
    };
    Ok(())
}

pub fn try_disconnect_http_proxy(proxy_principal: Principal) -> Result<(), HttpOverWsError> {
    STATE.with(|state| state.borrow_mut().remove_proxy(&proxy_principal))?;

    log(&format!(
        "http_over_ws: Client {} disconnected",
        proxy_principal
    ));
    Ok(())
}

fn handle_http_result(
    proxy_principal: Principal,
    request_id: HttpRequestId,
    http_result: HttpResult,
) {
    let res = STATE.with(|state| {
        state
            .borrow_mut()
            .update_connection_state(proxy_principal, request_id, http_result)
    });

    log(&format!(
        "http_over_ws: handled http result {:?} for request with id: {}",
        res, request_id
    ));
}

pub fn execute_http_request(
    req: HttpRequest,
    callback: Option<HttpCallback>,
    timeout_ms: Option<HttpRequestTimeoutMs>,
    ws_send: fn(Principal, Vec<u8>) -> Result<(), String>,
) -> ExecuteHttpRequestResult {
    let (assigned_proxy_principal, request_id) = STATE.with(|state| {
        state
            .borrow_mut()
            .assign_connection(req.clone(), callback, timeout_ms)
    })?;

    ws_send(
        assigned_proxy_principal,
        HttpOverWsMessage::HttpRequest(request_id, req).to_bytes(),
    )
    .unwrap();

    Ok(request_id)
}

pub fn get_http_connection(request_id: HttpRequestId) -> Option<HttpRequest> {
    STATE.with(|state| state.borrow().get_http_connection(request_id))
}

pub fn get_http_response(request_id: HttpRequestId) -> GetHttpResponseResult {
    STATE.with(|state| state.borrow().get_http_response(request_id))
}
