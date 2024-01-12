mod utils;

use candid::Nat;
use http_over_ws::{
    HttpFailureReason, HttpMethod, HttpOverWsError, HttpOverWsMessage, HttpRequest, HttpResponse,
};
use utils::{
    actor::CanisterActor, constants::TEST_HTTP_REQUEST_HEADER, ic_env, proxy_client::ProxyClient,
};

use crate::utils::constants::{TEST_HTTP_RESPONSE_HEADER, TEST_URL};

#[test]
fn test_execute_http_request_no_proxies_connected() {
    let test_env = ic_env::get_test_env();
    test_env.reset_canister();
    let canister_actor = CanisterActor::new(&test_env);

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let res = canister_actor.call_execute_http_request(request.clone(), None, false);

    assert_eq!(
        res,
        Err(HttpOverWsError::InvalidHttpMessage(
            HttpFailureReason::ProxyError(String::from("no proxies connected")),
        )),
    );
}

#[test]
fn test_execute_http_request_after_proxy_disconnected() {
    let test_env = ic_env::get_test_env();
    test_env.reset_canister();
    let mut proxy_proxy = ProxyClient::new(&test_env);
    let canister_actor = CanisterActor::new(&test_env);

    proxy_proxy.setup_proxy();
    proxy_proxy.close_ws_connection();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let res = canister_actor.call_execute_http_request(request, None, false);

    assert_eq!(
        res,
        Err(HttpOverWsError::InvalidHttpMessage(
            HttpFailureReason::ProxyError(String::from("no proxies connected")),
        )),
    );
}

#[test]
fn test_execute_http_request_without_response() {
    let test_env = ic_env::get_test_env();
    test_env.reset_canister();
    let mut proxy_proxy = ProxyClient::new(&test_env);
    let canister_actor = CanisterActor::new(&test_env);

    proxy_proxy.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let connection_id = canister_actor
        .call_execute_http_request(request.clone(), None, false)
        .unwrap();

    let proxy_messages = proxy_proxy.get_http_over_ws_messages();
    assert_eq!(
        proxy_messages[0],
        HttpOverWsMessage::HttpRequest(connection_id, request),
    );

    let http_response = canister_actor.query_get_http_response(connection_id);
    assert_eq!(http_response, Err(HttpFailureReason::NotYetReceived));
}

#[test]
fn test_execute_http_request() {
    let test_env = ic_env::get_test_env();
    test_env.reset_canister();
    let mut proxy_proxy = ProxyClient::new(&test_env);
    let canister_actor = CanisterActor::new(&test_env);

    proxy_proxy.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let connection_id = canister_actor
        .call_execute_http_request(request.clone(), None, false)
        .unwrap();

    let proxy_messages = proxy_proxy.get_http_over_ws_messages();
    assert_eq!(
        proxy_messages[0],
        HttpOverWsMessage::HttpRequest(connection_id, request),
    );

    let http_response = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };
    proxy_proxy.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        connection_id,
        http_response.clone(),
    ));

    let res = canister_actor.query_get_http_response(connection_id);
    assert_eq!(res, Ok(http_response));

    let callback_res = canister_actor.query_get_callback_responses();
    assert_eq!(callback_res.len(), 0);
}

#[test]
fn test_execute_http_request_with_body() {
    let test_env = ic_env::get_test_env();
    test_env.reset_canister();
    let mut proxy_proxy = ProxyClient::new(&test_env);
    let canister_actor = CanisterActor::new(&test_env);

    proxy_proxy.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        Some(vec![1, 2, 3]),
    );

    let connection_id = canister_actor
        .call_execute_http_request(request.clone(), None, false)
        .unwrap();

    let proxy_messages = proxy_proxy.get_http_over_ws_messages();
    assert_eq!(
        proxy_messages[0],
        HttpOverWsMessage::HttpRequest(connection_id, request),
    );

    let http_response = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };
    proxy_proxy.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        connection_id,
        http_response.clone(),
    ));

    let res = canister_actor.query_get_http_response(connection_id);
    assert_eq!(res, Ok(http_response));
}

#[test]
fn test_execute_http_request_with_proxy_error() {
    let test_env = ic_env::get_test_env();
    test_env.reset_canister();
    let mut proxy_proxy = ProxyClient::new(&test_env);
    let canister_actor = CanisterActor::new(&test_env);

    proxy_proxy.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let connection_id = canister_actor
        .call_execute_http_request(request.clone(), None, false)
        .unwrap();

    let proxy_messages = proxy_proxy.get_http_over_ws_messages();
    assert_eq!(
        proxy_messages[0],
        HttpOverWsMessage::HttpRequest(connection_id, request),
    );

    let error_message = String::from("proxy error");

    proxy_proxy.send_http_over_ws_message(HttpOverWsMessage::Error(
        Some(connection_id),
        error_message.clone(),
    ));

    let res = canister_actor.query_get_http_response(connection_id);
    assert_eq!(res, Err(HttpFailureReason::ProxyError(format!("http_over_ws: incoming error: {}", error_message))));
}

#[test]
fn test_execute_http_request_only_assigned_proxy() {
    let test_env = ic_env::get_test_env();
    test_env.reset_canister();
    let mut proxy_proxy1 = ProxyClient::new(&test_env);
    let mut proxy_proxy2 = ProxyClient::new(&test_env);
    let canister_actor = CanisterActor::new(&test_env);

    proxy_proxy1.setup_proxy();
    proxy_proxy2.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let connection_id = canister_actor
        .call_execute_http_request(request.clone(), None, false)
        .unwrap();

    // discover to which proxy the connection was assigned
    let proxy1_messages = proxy_proxy1.get_http_over_ws_messages();
    let proxy2_messages = proxy_proxy2.get_http_over_ws_messages();
    assert!(proxy1_messages.len() != proxy2_messages.len());

    let (mut assigned_proxy, mut idle_proxy) = if proxy1_messages.len() > 0 {
        (proxy_proxy1, proxy_proxy2)
    } else {
        (proxy_proxy2, proxy_proxy1)
    };

    let http_response = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };

    // test that the canister doesn't trap or break the state
    // if the response comes from an unassigned proxy
    idle_proxy.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        connection_id,
        http_response.clone(),
    ));
    let res = canister_actor.query_get_http_response(connection_id);
    assert_eq!(res, Err(HttpFailureReason::NotYetReceived));

    assigned_proxy.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        connection_id,
        http_response.clone(),
    ));

    let res = canister_actor.query_get_http_response(connection_id);
    assert_eq!(res, Ok(http_response));
}

#[test]
fn test_execute_http_request_multiple() {
    let test_env = ic_env::get_test_env();
    test_env.reset_canister();
    let mut proxy_proxy = ProxyClient::new(&test_env);
    let canister_actor = CanisterActor::new(&test_env);

    proxy_proxy.setup_proxy();

    let request1 = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );
    let request2 = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let connection_id1 = canister_actor
        .call_execute_http_request(request1.clone(), None, false)
        .unwrap();
    let connection_id2 = canister_actor
        .call_execute_http_request(request2.clone(), None, false)
        .unwrap();

    let proxy_messages = proxy_proxy.get_http_over_ws_messages();
    assert_eq!(
        proxy_messages[0],
        HttpOverWsMessage::HttpRequest(connection_id1, request1),
    );
    assert_eq!(
        proxy_messages[1],
        HttpOverWsMessage::HttpRequest(connection_id2, request2),
    );

    let http_response1 = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };
    let http_response2 = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };

    proxy_proxy.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        connection_id1,
        http_response1.clone(),
    ));
    proxy_proxy.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        connection_id2,
        http_response2.clone(),
    ));

    let res1 = canister_actor.query_get_http_response(connection_id1);
    assert_eq!(res1, Ok(http_response1));
    let res2 = canister_actor.query_get_http_response(connection_id2);
    assert_eq!(res2, Ok(http_response2));
}

#[test]
fn test_execute_http_request_before_timeout() {
    let test_env = ic_env::get_test_env();
    test_env.reset_canister();
    let mut proxy_proxy = ProxyClient::new(&test_env);
    let canister_actor = CanisterActor::new(&test_env);

    proxy_proxy.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let connection_id = canister_actor
        .call_execute_http_request(request.clone(), Some(10_000), false)
        .unwrap();

    let proxy_messages = proxy_proxy.get_http_over_ws_messages();
    assert_eq!(
        proxy_messages[0],
        HttpOverWsMessage::HttpRequest(connection_id, request),
    );

    // make some time pass, but not enough to trigger the timeout
    test_env.advance_canister_time_ms(8_000);

    let http_response = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };
    proxy_proxy.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        connection_id,
        http_response.clone(),
    ));

    let res = canister_actor.query_get_http_response(connection_id);
    assert_eq!(res, Ok(http_response));
}

#[test]
fn test_execute_http_request_timeout_expired() {
    let test_env = ic_env::get_test_env();
    test_env.reset_canister();
    let mut proxy_proxy = ProxyClient::new(&test_env);
    let canister_actor = CanisterActor::new(&test_env);

    proxy_proxy.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let connection_id = canister_actor
        .call_execute_http_request(request.clone(), Some(10_000), false)
        .unwrap();

    let proxy_messages = proxy_proxy.get_http_over_ws_messages();
    assert_eq!(
        proxy_messages[0],
        HttpOverWsMessage::HttpRequest(connection_id, request),
    );

    // advance time so that the timeout expires
    test_env.advance_canister_time_ms(10_000);

    let res = canister_actor.query_get_http_response(connection_id);
    assert_eq!(res, Err(HttpFailureReason::RequestTimeout));

    // even after sending the response,
    // the connection shouldn't change its state
    let http_response = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };
    proxy_proxy.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        connection_id,
        http_response.clone(),
    ));

    let res = canister_actor.query_get_http_response(connection_id);
    assert_eq!(res, Err(HttpFailureReason::RequestTimeout));
}

#[test]
fn test_execute_http_request_with_callback() {
    let test_env = ic_env::get_test_env();
    test_env.reset_canister();
    let mut proxy_proxy = ProxyClient::new(&test_env);
    let canister_actor = CanisterActor::new(&test_env);

    proxy_proxy.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let connection_id = canister_actor
        .call_execute_http_request(request.clone(), None, true)
        .unwrap();

    let proxy_messages = proxy_proxy.get_http_over_ws_messages();
    assert_eq!(
        proxy_messages[0],
        HttpOverWsMessage::HttpRequest(connection_id, request),
    );

    let http_response = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };
    proxy_proxy.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        connection_id,
        http_response.clone(),
    ));

    let res = canister_actor.query_get_http_response(connection_id);
    assert_eq!(res, Ok(http_response.clone()));

    let callback_res = canister_actor.query_get_callback_responses();
    assert_eq!(callback_res.len(), 1);
    assert_eq!(callback_res[0], http_response);
}

#[test]
fn test_execute_http_request_duplicate_response() {
    let test_env = ic_env::get_test_env();
    test_env.reset_canister();
    let mut proxy_proxy = ProxyClient::new(&test_env);
    let canister_actor = CanisterActor::new(&test_env);

    proxy_proxy.setup_proxy();

    let request = HttpRequest::new(
        TEST_URL,
        HttpMethod::GET,
        vec![TEST_HTTP_REQUEST_HEADER.clone()],
        None,
    );

    let connection_id = canister_actor
        .call_execute_http_request(request.clone(), None, true)
        .unwrap();

    let proxy_messages = proxy_proxy.get_http_over_ws_messages();
    assert_eq!(
        proxy_messages[0],
        HttpOverWsMessage::HttpRequest(connection_id, request),
    );

    let http_response1 = HttpResponse {
        status: Nat::from(200),
        headers: vec![TEST_HTTP_RESPONSE_HEADER.clone()],
        body: vec![1, 2, 3],
    };
    proxy_proxy.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        connection_id,
        http_response1.clone(),
    ));

    let res = canister_actor.query_get_http_response(connection_id);
    assert_eq!(res, Ok(http_response1.clone()));
    let callback_res = canister_actor.query_get_callback_responses();
    assert_eq!(callback_res.len(), 1);
    assert_eq!(callback_res[0], http_response1);

    // sending another response again should not change the state
    // and hence not invoke the callback again
    let http_response2 = HttpResponse {
        status: Nat::from(400),
        headers: vec![],
        body: vec![4, 5, 6],
    };
    proxy_proxy.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        connection_id,
        http_response2.clone(),
    ));

    let res = canister_actor.query_get_http_response(connection_id);
    assert_eq!(res, Ok(http_response1.clone()));
    let callback_res = canister_actor.query_get_callback_responses();
    assert_eq!(callback_res.len(), 1);
    assert_eq!(callback_res[0], http_response1);
}

#[test]
fn test_get_http_response_not_found() {
    let test_env = ic_env::get_test_env();
    test_env.reset_canister();
    let canister_actor = CanisterActor::new(&test_env);

    let res = canister_actor.query_get_http_response(0);
    assert_eq!(res, Err(HttpFailureReason::ConnectionIdNotFound));
}
