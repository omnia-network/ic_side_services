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
            Ok(())
        }
        HttpOverWsMessage::HttpResponse(request_id, response) => {
            handle_http_response(proxy_principal, request_id, response)
        }
        HttpOverWsMessage::Error(request_id, err) => {
            handle_http_error(proxy_principal, request_id, err)
        }
        HttpOverWsMessage::HttpRequest(_, _) => {
            let e = String::from(
                "http_over_ws: client proxy is not allowed to send HTTP connections over WS",
            );
            log(&e);
            Err(HttpOverWsError::InvalidHttpMessage(
                HttpFailureReason::ProxyError(e),
            ))
        }
    }
}

pub fn try_disconnect_http_proxy(proxy_principal: Principal) -> Result<(), HttpFailureReason> {
    STATE.with(|state| state.borrow_mut().remove_proxy(&proxy_principal))?;

    log(&format!(
        "http_over_ws: Client {} disconnected",
        proxy_principal
    ));
    Ok(())
}

fn handle_http_response(
    proxy_principal: Principal,
    request_id: HttpRequestId,
    response: HttpResponse,
) -> Result<(), HttpOverWsError> {
    STATE.with(|state| {
        state
            .borrow_mut()
            .update_connection_state(proxy_principal, request_id, HttpResult::Success(response))
            .map_err(|e| HttpOverWsError::InvalidHttpMessage(e))
    })?;

    log(&format!(
        "http_over_ws: completed HTTP connection {}",
        request_id
    ));

    Ok(())
}

fn handle_http_error(
    proxy_principal: Principal,
    request_id: Option<HttpRequestId>,
    err: String,
) -> Result<(), HttpOverWsError> {
    log(&err);

    let reason = HttpFailureReason::ProxyError(err);

    if let Some(request_id) = request_id {
        STATE.with(|state| {
            state
                .borrow_mut()
                .update_connection_state(proxy_principal, request_id, HttpResult::Failure(reason.clone()))
                .map_err(|e| HttpOverWsError::InvalidHttpMessage(e))
        })?;
    }
    Err(HttpOverWsError::InvalidHttpMessage(reason))
}

pub fn execute_http_request(
    req: HttpRequest,
    callback: Option<HttpCallback>,
    timeout_ms: Option<HttpRequestTimeoutMs>,
    ws_send: fn(Principal, Vec<u8>) -> Result<(), String>,
) -> ExecuteHttpRequestResult {
    let (assigned_proxy_principal, request_id) = STATE
        .with(|state| {
            state
                .borrow_mut()
                .assign_connection(req.clone(), callback, timeout_ms)
        })
        .map_err(|e| HttpOverWsError::InvalidHttpMessage(e))?;

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
