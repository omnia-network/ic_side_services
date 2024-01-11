use std::time::Duration;

use crate::{types::*, CONNECTED_CLIENTS, HTTP_REQUESTS};
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
            CONNECTED_CLIENTS.with(|clients| {
                clients.borrow_mut().add_client(client_principal);
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
            log(&err);

            if let Some(request_id) = request_id {
                HTTP_REQUESTS.with(|http_requests| {
                    http_requests
                        .borrow_mut()
                        .get_mut(&request_id)
                        .and_then(|r| {
                            r.failure_reason = Some(HttpFailureReason::ProxyError(err));
                            Some(r)
                        });
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
    CONNECTED_CLIENTS.with(|clients| clients.borrow_mut().remove_client(&client_principal))?;

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
    CONNECTED_CLIENTS.with(|clients| {
        clients
            .borrow_mut()
            .complete_request_for_client(client_principal, request_id)
            .map_err(|e| HttpOverWsError::InvalidHttpMessage(e))
    })?;

    // assign response to a previous request
    HTTP_REQUESTS.with(|http_requests| -> Result<(), HttpOverWsError> {
        let mut h = http_requests.borrow_mut();
        let r = h
            .get_mut(&request_id)
            .ok_or(HttpOverWsError::InvalidHttpMessage(
                HttpFailureReason::RequestIdNotFound,
            ))?;
        r.response = Some(response.clone());

        // response has been received, clear the timer if it was set
        if let Some(timer_id) = r.timer_id.take() {
            ic_cdk_timers::clear_timer(timer_id);
        }

        // if a callback was set, execute it
        if let Some(callback) = r.callback {
            ic_cdk::spawn(async move { callback(response).await });
        }

        Ok(())
    })?;

    log(&format!(
        "http_over_ws: completed HTTP request {}",
        request_id
    ));

    Ok(())
}

fn http_request_timeout(client_principal: Principal, request_id: HttpRequestId) {
    if let Err(_) = CONNECTED_CLIENTS.with(|clients| {
        clients
            .borrow_mut()
            .complete_request_for_client(client_principal, request_id)
    }) {
        log("cannot complete request");
    }

    HTTP_REQUESTS.with(|http_requests| {
        http_requests
            .borrow_mut()
            .get_mut(&request_id)
            .and_then(|r| {
                if r.response.is_none() {
                    r.failure_reason = Some(HttpFailureReason::RequestTimeout);

                    log(&format!(
                        "http_over_ws: HTTP request with id {} timed out",
                        request_id
                    ));
                }

                Some(r)
            });
    });
}

pub fn execute_http_request(
    req: HttpRequest,
    callback: Option<HttpCallback>,
    timeout_ms: Option<HttpRequestTimeoutMs>,
    ws_send: fn(Principal, Vec<u8>) -> Result<(), String>,
) -> ExecuteHttpRequestResult {
    let request_id = HTTP_REQUESTS.with(|http_requests| http_requests.borrow().len() + 1) as u32;

    let assigned_client_principal = CONNECTED_CLIENTS
        .with(|clients| clients.borrow_mut().assign_request(request_id))
        .map_err(|e| HttpOverWsError::InvalidHttpMessage(e))?;

    let timer_id = timeout_ms.and_then(|millis| {
        Some(ic_cdk_timers::set_timer(
            Duration::from_millis(millis),
            move || {
                http_request_timeout(assigned_client_principal, request_id);
            },
        ))
    });

    HTTP_REQUESTS.with(|http_requests| {
        http_requests.borrow_mut().insert(
            request_id,
            HttpRequestState::new(req.clone(), callback, timer_id),
        );
    });

    ws_send(
        assigned_client_principal,
        HttpOverWsMessage::HttpRequest(request_id, req).to_bytes(),
    )
    .unwrap();

    Ok(request_id)
}

pub fn get_http_request(request_id: HttpRequestId) -> Option<HttpRequest> {
    HTTP_REQUESTS.with(|http_requests| {
        http_requests
            .borrow()
            .get(&request_id)
            .map(|r| r.request.to_owned())
    })
}

pub fn get_http_response(request_id: HttpRequestId) -> GetHttpResponseResult {
    HTTP_REQUESTS.with(|http_requests| {
        http_requests
            .borrow()
            .get(&request_id)
            .ok_or(HttpFailureReason::RequestIdNotFound)
            .map(|r| {
                r.response
                    .as_ref()
                    .ok_or(
                        r.failure_reason
                            .clone()
                            .unwrap_or(HttpFailureReason::Unknown),
                    )
                    .cloned()
            })?
    })
}
