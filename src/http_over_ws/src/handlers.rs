use crate::{types::*, STATE};
use candid::Principal;
use logger::log;

/// Called by the callback passed to the IC WS cdk when a new message is received.
/// Checks if the message is an [HttpOverWsMessage], and if so it handles it.
/// Otherwise, it returns [`HttpOverWsError::NotHttpOverWsType`] to signal the callback that it should treat it as a WS message sent by one of the WS clients (and not an HTTP Proxy)
pub fn try_handle_http_over_ws_message(
    client_principal: Principal,
    serialized_message: Vec<u8>,
) -> Result<(), HttpOverWsError> {
    let incoming_msg = HttpOverWsMessage::from_bytes(&serialized_message)
        .map_err(|e| HttpOverWsError::NotHttpOverWsType(e))?;

    log(&format!(
        "http_over_ws: incoming message: {:?} from {}",
        incoming_msg, client_principal
    ));

    match incoming_msg {
        HttpOverWsMessage::SetupProxyClient => {
            STATE.with(|state| {
                state.borrow_mut().add_client(client_principal);
            });
            log(&format!(
                "http_over_ws: proxy client {} connected",
                client_principal
            ));
            Ok(())
        }
        HttpOverWsMessage::HttpResponse(request_id, response) => {
            handle_http_response(client_principal, request_id, response)
        }
        HttpOverWsMessage::Error(request_id, err) => {
            let e = format!("http_over_ws: incoming error: {}", err);
            log(&e);

            if let Some(request_id) = request_id {
                STATE.with(|state| {
                    state
                        .borrow_mut()
                        .report_http_failure(client_principal, request_id, HttpFailureReason::ProxyError(e.clone()));
                        
                });
            }
            Err(HttpOverWsError::InvalidHttpMessage(
                HttpFailureReason::ProxyError(e),
            ))
        }
        HttpOverWsMessage::HttpRequest(_, _) => {
            let e = String::from(
                "http_over_ws: proxy client is not allowed to send HTTP requests over WS",
            );
            log(&e);
            Err(HttpOverWsError::InvalidHttpMessage(
                HttpFailureReason::ProxyError(e),
            ))
        }
    }
}

pub fn try_disconnect_http_proxy(client_principal: Principal) -> Result<(), HttpFailureReason> {
    STATE.with(|state| state.borrow_mut().remove_client(&client_principal))?;

    log(&format!(
        "http_over_ws: Client {} disconnected",
        client_principal
    ));
    Ok(())
}

fn handle_http_response(
    client_principal: Principal,
    request_id: HttpRequestId,
    response: HttpResponse,
) -> Result<(), HttpOverWsError> {
    STATE.with(|state| {
        state
            .borrow_mut()
            .handle_http_response(client_principal, request_id, response)
            .map_err(|e| HttpOverWsError::InvalidHttpMessage(e))
    })?;

    log(&format!(
        "http_over_ws: completed HTTP request {}",
        request_id
    ));

    Ok(())
}

pub fn execute_http_request(
    req: HttpRequest,
    callback: Option<HttpCallback>,
    timeout_ms: Option<HttpRequestTimeoutMs>,
    ws_send: fn(Principal, Vec<u8>) -> Result<(), String>,
) -> ExecuteHttpRequestResult {
    let (assigned_client_principal, request_id) = STATE
        .with(|state| state.borrow_mut().assign_request(req.clone(), callback, timeout_ms))
        .map_err(|e| HttpOverWsError::InvalidHttpMessage(e))?;

    ws_send(
        assigned_client_principal,
        HttpOverWsMessage::HttpRequest(request_id, req).to_bytes(),
    )
    .unwrap();

    Ok(request_id)
}

pub fn get_http_request(request_id: HttpRequestId) -> Option<HttpRequest> {
    STATE.with(|state| {
        state.borrow().get_http_request(request_id)
    })
}

pub fn get_http_response(request_id: HttpRequestId) -> GetHttpResponseResult {
    STATE.with(|state| {
        state
            .borrow()
            .get_http_response(request_id)
    })
}