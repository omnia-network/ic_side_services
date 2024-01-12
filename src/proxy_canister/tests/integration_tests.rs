mod utils;

use std::{
    path::PathBuf,
    sync::{Mutex, Once},
};

use candid::{encode_args, Principal};
use http_over_ws::{HttpMethod, HttpRequest};
use lazy_static::lazy_static;
use proxy_canister_types::HttpRequestEndpointArgs;
use test_utils::{
    ic_env::{get_test_env, load_canister_wasm_from_path, CanisterData},
    identity::generate_random_principal,
    proxy_client::ProxyClient,
};
use utils::{actors::TestUserCanisterActor, constants::TEST_URL};

lazy_static! {
    static ref TEST_USER_CANISTER_WASM_MODULE: Vec<u8> =
        load_canister_wasm_from_path(&PathBuf::from(
            std::env::var("TEST_USER_CANISTER_WASM_PATH")
                .expect("TEST_USER_CANISTER_WASM_PATH must be set")
        ));
    static ref PROXY_CANISTER_WASM_MODULE: Vec<u8> = load_canister_wasm_from_path(&PathBuf::from(
        std::env::var("PROXY_CANISTER_WASM_PATH").expect("PROXY_CANISTER_WASM_PATH must be set")
    ));
    static ref TEST_USER_CANISTER_CONTROLLER: Principal = generate_random_principal();
    static ref TEST_USER_CANISTER_ID: Mutex<Principal> = Mutex::new(Principal::anonymous());
    static ref PROXY_CANISTER_ID: Mutex<Principal> = Mutex::new(Principal::anonymous());
}

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        let mut test_env = get_test_env();

        let proxy_canister_id = test_env.add_canister(CanisterData {
            wasm_module: PROXY_CANISTER_WASM_MODULE.clone(),
            args: vec![],
            controller: None,
        });

        let mut m = PROXY_CANISTER_ID.lock().unwrap();
        *m = proxy_canister_id;

        let test_user_canister_id = test_env.add_canister(CanisterData {
            wasm_module: TEST_USER_CANISTER_WASM_MODULE.clone(),
            args: encode_args((proxy_canister_id,)).unwrap(),
            controller: Some(*TEST_USER_CANISTER_CONTROLLER),
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

    proxy_client.expect_received_http_requests_count(1);

    assert!(res.is_ok());
}
