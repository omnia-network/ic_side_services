mod utils;

use std::{
    path::PathBuf,
    sync::{Mutex, Once},
};

use candid::{encode_args, Nat, Principal};
use http_over_ws::{HttpHeader, HttpMethod, HttpOverWsMessage, HttpRequest, HttpResponse};
use lazy_static::lazy_static;
use pocket_ic::{ErrorCode, UserError};
use proxy_canister_types::{HttpRequestEndpointArgs, InvalidRequest, ProxyError, RequestState};
use test_utils::{
    ic_env::{get_test_env, load_canister_wasm_from_path, CanisterData},
    identity::generate_random_principal,
    proxy_client::ProxyClient,
};
use utils::{
    actors::{ProxyCanisterActor, TestUserCanisterActor},
    constants::TEST_URL,
};

static TEST_USER_CANISTER_ID: Mutex<Principal> = Mutex::new(Principal::anonymous());
static PROXY_CANISTER_ID: Mutex<Principal> = Mutex::new(Principal::anonymous());

lazy_static! {
    static ref TEST_USER_CANISTER_WASM_MODULE: Vec<u8> =
        load_canister_wasm_from_path(&PathBuf::from(
            std::env::var("TEST_USER_CANISTER_WASM_PATH")
                .expect("TEST_USER_CANISTER_WASM_PATH must be set")
        ));
    static ref PROXY_CANISTER_WASM_MODULE: Vec<u8> = load_canister_wasm_from_path(&PathBuf::from(
        std::env::var("PROXY_CANISTER_WASM_PATH").expect("PROXY_CANISTER_WASM_PATH must be set")
    ));
    static ref PROXY_CANISTER_CONTROLLER: Principal = generate_random_principal();
}

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        let mut test_env = get_test_env();

        let proxy_canister_id = test_env.add_canister(CanisterData {
            wasm_module: PROXY_CANISTER_WASM_MODULE.clone(),
            args: encode_args(()).unwrap(),
            controller: Some(*PROXY_CANISTER_CONTROLLER),
        });

        let mut m = PROXY_CANISTER_ID.lock().unwrap();
        *m = proxy_canister_id;

        let test_user_canister_id = test_env.add_canister(CanisterData {
            wasm_module: TEST_USER_CANISTER_WASM_MODULE.clone(),
            args: encode_args((proxy_canister_id,)).unwrap(),
            controller: None,
        });

        let mut m = TEST_USER_CANISTER_ID.lock().unwrap();
        *m = test_user_canister_id;
    });
}

fn reset_canisters() {
    let test_env = get_test_env();
    test_env
        .get_canisters()
        .into_keys()
        .for_each(|canister_id| {
            test_env.reset_canister(&canister_id);
        });
}

fn get_test_user_canister_id() -> Principal {
    TEST_USER_CANISTER_ID.lock().unwrap().clone()
}

fn get_proxy_canister_id() -> Principal {
    PROXY_CANISTER_ID.lock().unwrap().clone()
}

#[test]
fn test_proxy_canister_http_request_anonymous() {
    setup();
    reset_canisters();
    let test_env = get_test_env();
    let proxy_canister_id = get_proxy_canister_id();
    let proxy_canister_actor = ProxyCanisterActor::new(&test_env, proxy_canister_id);

    let res = proxy_canister_actor.call_http_request(
        Principal::anonymous(),
        HttpRequestEndpointArgs {
            request: HttpRequest {
                url: TEST_URL.to_string(),
                method: HttpMethod::GET,
                headers: vec![],
                body: None,
            },
            timeout_ms: None,
            callback_method_name: None,
        },
    );

    assert_eq!(
        res,
        Err(UserError {
            code: ErrorCode::CanisterCalledTrap,
            description: format!(
                "Canister {} trapped explicitly: Caller is anonymous",
                proxy_canister_id
            ),
        })
    )
}

#[test]
fn test_http_request_invalid() {
    setup();
    reset_canisters();
    let test_env = get_test_env();
    let mut proxy_client = ProxyClient::new(&test_env, get_proxy_canister_id());
    let test_canister_actor = TestUserCanisterActor::new(&test_env, get_test_user_canister_id());

    proxy_client.setup_proxy();

    // invalid url
    let res = test_canister_actor.call_http_request_via_proxy(HttpRequestEndpointArgs {
        request: HttpRequest {
            url: String::from("invalid url"),
            method: HttpMethod::GET,
            headers: vec![],
            body: None,
        },
        timeout_ms: None,
        callback_method_name: None,
    });
    assert_eq!(
        res,
        Err(ProxyError::InvalidRequest(InvalidRequest::InvalidUrl(
            "relative URL without a base".to_string()
        ))),
    );
    proxy_client.expect_received_http_requests_count(0);

    // too many headers
    let res = test_canister_actor.call_http_request_via_proxy(HttpRequestEndpointArgs {
        request: HttpRequest {
            url: TEST_URL.to_string(),
            method: HttpMethod::GET,
            // more headers than the maximum allowed
            headers: (0..60)
                .map(|i| HttpHeader {
                    name: format!("name_{}", i),
                    value: format!("value_{}", i),
                })
                .collect(),
            body: None,
        },
        timeout_ms: None,
        callback_method_name: None,
    });
    assert_eq!(
        res,
        Err(ProxyError::InvalidRequest(InvalidRequest::TooManyHeaders))
    );
    proxy_client.expect_received_http_requests_count(0);

    // invalid timeouts
    let res = test_canister_actor.call_http_request_via_proxy(HttpRequestEndpointArgs {
        request: HttpRequest {
            url: TEST_URL.to_string(),
            method: HttpMethod::GET,
            headers: vec![],
            body: None,
        },
        timeout_ms: Some(0), // less than the min
        callback_method_name: None,
    });
    assert_eq!(
        res,
        Err(ProxyError::InvalidRequest(InvalidRequest::InvalidTimeout)),
    );
    proxy_client.expect_received_http_requests_count(0);

    let res = test_canister_actor.call_http_request_via_proxy(HttpRequestEndpointArgs {
        request: HttpRequest {
            url: TEST_URL.to_string(),
            method: HttpMethod::GET,
            headers: vec![],
            body: None,
        },
        timeout_ms: Some(70_000), // more than the max
        callback_method_name: None,
    });
    assert_eq!(
        res,
        Err(ProxyError::InvalidRequest(InvalidRequest::InvalidTimeout)),
    );
    proxy_client.expect_received_http_requests_count(0);
}

fn test_wrong_callback(callback_name: &str) {
    let test_env = get_test_env();
    let mut proxy_client = ProxyClient::new(&test_env, get_proxy_canister_id());
    let test_canister_id = get_test_user_canister_id();
    let test_canister_actor = TestUserCanisterActor::new(&test_env, test_canister_id);
    let proxy_canister_actor = ProxyCanisterActor::new(&test_env, get_proxy_canister_id());

    proxy_client.setup_proxy();

    println!("testing callback: {}", callback_name);

    let request_id1 = test_canister_actor
        .call_http_request_via_proxy(HttpRequestEndpointArgs {
            request: HttpRequest {
                url: TEST_URL.to_string(),
                method: HttpMethod::GET,
                headers: vec![],
                body: None,
            },
            timeout_ms: None,
            callback_method_name: Some(callback_name.to_string()),
        })
        .unwrap();

    proxy_client.expect_received_http_requests_count(1);

    proxy_client.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        request_id1,
        HttpResponse {
            status: Nat::from(200),
            headers: vec![],
            body: vec![1, 2, 3],
        },
    ));

    test_env.tick_n(10);

    let req_state1 = proxy_canister_actor
        .query_get_request_by_id_with_panic(*PROXY_CANISTER_CONTROLLER, request_id1)
        .unwrap();
    assert_eq!(req_state1.canister_id, test_canister_id);
    assert!(matches!(req_state1.state, RequestState::Failed(_)));

    // the proxy canister shouldn't trap and hence not break the ws connection
    // so we try to send another request
    let req_2 = HttpRequest {
        url: TEST_URL.to_string(),
        method: HttpMethod::GET,
        headers: vec![],
        body: None,
    };
    let request_id2 = test_canister_actor
        .call_http_request_via_proxy(HttpRequestEndpointArgs {
            request: req_2.clone(),
            timeout_ms: None,
            callback_method_name: Some(callback_name.to_string()),
        })
        .unwrap();

    test_env.tick_n(10);

    // we expect to receive the new request
    let proxy_messages = proxy_client.get_http_over_ws_messages();
    assert_eq!(
        proxy_messages,
        vec![HttpOverWsMessage::HttpRequest(request_id2, req_2,)]
    );
    // send the response expecting that the ws connection still works
    proxy_client.send_http_over_ws_message(HttpOverWsMessage::HttpResponse(
        request_id2,
        HttpResponse {
            status: Nat::from(200),
            headers: vec![],
            body: vec![1, 2, 3],
        },
    ));

    let req_state2 = proxy_canister_actor
        .query_get_request_by_id_with_panic(*PROXY_CANISTER_CONTROLLER, request_id2)
        .unwrap();
    assert_eq!(req_state2.canister_id, test_canister_id);
    assert!(matches!(req_state2.state, RequestState::Failed(_)));
}

#[test]
fn test_http_request_wrong_callbacks() {
    setup();
    reset_canisters();

    test_wrong_callback("wrong_callback_method");
    test_wrong_callback("http_response_callback_traps");
    test_wrong_callback("http_response_callback_wrong_args");
}

#[test]
fn test_http_request() {
    setup();
    reset_canisters();
    let test_env = get_test_env();
    let mut proxy_client = ProxyClient::new(&test_env, get_proxy_canister_id());
    let test_canister_actor = TestUserCanisterActor::new(&test_env, get_test_user_canister_id());

    proxy_client.setup_proxy();

    let res = test_canister_actor.call_http_request_via_proxy(HttpRequestEndpointArgs {
        request: HttpRequest {
            url: TEST_URL.to_string(),
            method: HttpMethod::GET,
            headers: vec![],
            body: None,
        },
        timeout_ms: None,
        callback_method_name: None,
    });

    assert!(res.is_ok());

    proxy_client.expect_received_http_requests_count(1);
}

#[test]
fn test_http_request_with_timeout() {
    setup();
    reset_canisters();
    let test_env = get_test_env();
    let mut proxy_client = ProxyClient::new(&test_env, get_proxy_canister_id());
    let test_canister_actor = TestUserCanisterActor::new(&test_env, get_test_user_canister_id());

    proxy_client.setup_proxy();

    let timeout_ms = 10_000;

    let res = test_canister_actor.call_http_request_via_proxy(HttpRequestEndpointArgs {
        request: HttpRequest {
            url: TEST_URL.to_string(),
            method: HttpMethod::GET,
            headers: vec![],
            body: None,
        },
        timeout_ms: Some(timeout_ms),
        callback_method_name: None,
    });
    assert!(res.is_ok());
    proxy_client.expect_received_http_requests_count(1);
}

#[test]
fn test_get_request_by_id_unauthorized() {
    setup();
    reset_canisters();
    let test_env = get_test_env();
    let proxy_canister_id = get_proxy_canister_id();
    let proxy_canister_actor = ProxyCanisterActor::new(&test_env, proxy_canister_id);

    let res = proxy_canister_actor.query_get_request_by_id(generate_random_principal(), 1);

    assert_eq!(
        res,
        Err(UserError {
            code: ErrorCode::CanisterCalledTrap,
            description: format!(
                "Canister {} trapped explicitly: Caller is not a controller",
                proxy_canister_id
            ),
        })
    )
}

#[test]
fn test_get_logs_unauthorized() {
    setup();
    reset_canisters();
    let test_env = get_test_env();
    let proxy_canister_id = get_proxy_canister_id();
    let proxy_canister_actor = ProxyCanisterActor::new(&test_env, proxy_canister_id);

    let res = proxy_canister_actor.query_get_logs(generate_random_principal());

    assert_eq!(
        res,
        Err(UserError {
            code: ErrorCode::CanisterCalledTrap,
            description: format!(
                "Canister {} trapped explicitly: Caller is not a controller",
                proxy_canister_id
            ),
        })
    )
}

#[test]
fn test_get_logs() {
    setup();
    reset_canisters();
    let test_env = get_test_env();
    let proxy_canister_id = get_proxy_canister_id();
    let test_canister_actor = TestUserCanisterActor::new(&test_env, get_test_user_canister_id());
    let proxy_canister_actor = ProxyCanisterActor::new(&test_env, proxy_canister_id);

    // execute an http request before to have logs
    let _ = test_canister_actor.call_http_request_via_proxy(HttpRequestEndpointArgs {
        request: HttpRequest {
            url: TEST_URL.to_string(),
            method: HttpMethod::GET,
            headers: vec![],
            body: None,
        },
        timeout_ms: None,
        callback_method_name: None,
    });

    let res = proxy_canister_actor.query_get_logs(*PROXY_CANISTER_CONTROLLER);
    assert!(res.unwrap().len() > 0);
}
