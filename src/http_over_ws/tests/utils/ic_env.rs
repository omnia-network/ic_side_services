use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
    time::{Duration, SystemTime},
};

use candid::{decode_one, encode_args, utils::ArgumentEncoder, CandidType, Deserialize, Principal};
use lazy_static::lazy_static;
use pocket_ic::{PocketIc, PocketIcBuilder, WasmResult};
use std::fs::File;
use std::io::Read;

use super::actor::CanisterMethod;

lazy_static! {
    pub static ref TEST_ENV: Mutex<TestEnv> = Mutex::new(TestEnv::new());
    static ref TEST_CANISTER_WASM_MODULE: Vec<u8> = load_canister_wasm_from_path(&PathBuf::from(
        std::env::var("TEST_CANISTER_WASM_PATH").expect("TEST_CANISTER_WASM_PATH must be set")
    ));
}

pub fn get_test_env<'a>() -> MutexGuard<'a, TestEnv> {
    TEST_ENV.lock().unwrap()
}

pub struct TestEnv {
    pub pic: PocketIc,
    test_canister_id: Principal,
}

impl TestEnv {
    pub fn new() -> Self {
        let pic = PocketIcBuilder::new()
            // NNS subnet needed to retrieve the root key
            .with_nns_subnet()
            .with_application_subnet()
            .build();

        // set ic time to current time
        pic.set_time(SystemTime::now());

        let app_subnet = pic.topology().get_app_subnets()[0];
        let canister_id = pic.create_canister_on_subnet(None, None, app_subnet);
        pic.add_cycles(canister_id, 1_000_000_000_000_000);

        pic.install_canister(
            canister_id,
            TEST_CANISTER_WASM_MODULE.clone(),
            candid::encode_args(()).unwrap(),
            None,
        );

        Self {
            pic,
            test_canister_id: canister_id,
        }
    }

    pub fn get_test_canister_id(&self) -> Principal {
        self.test_canister_id
    }

    pub fn reset_canister(&self) {
        self.tick_n(10);

        self.pic
            .reinstall_canister(
                self.test_canister_id,
                TEST_CANISTER_WASM_MODULE.clone(),
                candid::encode_args(()).unwrap(),
                None,
            )
            .unwrap();
    }

    /// Produce and advance by some blocks to fire eventual timers.
    ///
    /// See https://forum.dfinity.org/t/pocketic-multi-subnet-canister-testing/24901/4.
    pub fn tick_n(&self, n: u8) {
        for _ in 0..n {
            self.pic.tick();
        }
    }

    pub fn advance_canister_time_ms(&self, ms: u64) {
        self.pic.advance_time(Duration::from_millis(ms));
        self.tick_n(100);
    }

    /// # Panics
    /// if the call returns a [WasmResult::Reject].
    pub fn call_canister_method_with_panic<T, S>(
        &self,
        caller: Principal,
        method: CanisterMethod,
        args: T,
    ) -> S
    where
        T: CandidType + ArgumentEncoder,
        S: CandidType + for<'a> Deserialize<'a>,
    {
        let res = self
            .pic
            .update_call(
                self.get_test_canister_id(),
                caller,
                &method.to_string(),
                encode_args(args).unwrap(),
            )
            .expect("Failed to call canister");

        match res {
            WasmResult::Reply(bytes) => decode_one(&bytes).unwrap(),
            _ => panic!("Expected reply"),
        }
    }

    /// # Panics
    /// if the call returns a [WasmResult::Reject].
    pub fn query_canister_method_with_panic<T, S>(
        &self,
        caller: Principal,
        method: CanisterMethod,
        args: T,
    ) -> S
    where
        T: CandidType + ArgumentEncoder,
        S: CandidType + for<'a> Deserialize<'a>,
    {
        let res = self
            .pic
            .query_call(
                self.get_test_canister_id(),
                caller,
                &method.to_string(),
                encode_args(args).unwrap(),
            )
            .expect("Failed to call canister");

        match res {
            WasmResult::Reply(bytes) => decode_one(&bytes).unwrap(),
            _ => panic!("Expected reply"),
        }
    }
}

fn load_canister_wasm_from_path(path: &PathBuf) -> Vec<u8> {
    let mut file = File::open(&path)
        .unwrap_or_else(|_| panic!("Failed to open file: {}", path.to_str().unwrap()));
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("Failed to read file");
    bytes
}
