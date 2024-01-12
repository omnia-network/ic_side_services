use std::{path::PathBuf, sync::Once};

use candid::{encode_args, Principal};
use lazy_static::lazy_static;
use test_utils::{
    ic_env::{get_test_env, load_canister_wasm_from_path, CanisterData},
    identity::generate_random_principal,
};

lazy_static! {
    pub static ref TEST_USER_CANISTER_WASM_MODULE: Vec<u8> =
        load_canister_wasm_from_path(&PathBuf::from(
            std::env::var("TEST_USER_CANISTER_WASM_PATH")
                .expect("TEST_USER_CANISTER_WASM_PATH must be set")
        ));
    pub static ref PROXY_CANISTER_WASM_MODULE: Vec<u8> =
        load_canister_wasm_from_path(&PathBuf::from(
            std::env::var("PROXY_CANISTER_WASM_PATH")
                .expect("PROXY_CANISTER_WASM_PATH must be set")
        ));
    pub static ref TEST_USER_CANISTER_CONTROLLER: Principal = generate_random_principal();
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

        test_env.add_canister(CanisterData {
            wasm_module: TEST_USER_CANISTER_WASM_MODULE.clone(),
            args: encode_args((proxy_canister_id,)).unwrap(),
            controller: Some(*TEST_USER_CANISTER_CONTROLLER),
        });
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

#[test]
fn test() {
    setup();
    reset_canisters();
}
