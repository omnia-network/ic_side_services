use std::collections::HashMap;

use candid::Principal;
use http_over_ws::{HttpRequestId, HttpResponse};
use proxy_canister_types::{HttpRequestEndpointArgs, HttpRequestEndpointResult};
use test_utils::{ic_env::TestEnv, identity::generate_random_principal};

pub struct TestUserCanisterActor<'a> {
    test_env: &'a TestEnv,
    principal: Principal,
    canister_id: Principal,
}

impl<'a> TestUserCanisterActor<'a> {
    pub fn new(test_env: &'a TestEnv, canister_id: Principal) -> Self {
        Self {
            test_env,
            principal: generate_random_principal(),
            canister_id,
        }
    }

    pub fn call_http_request_via_proxy(
        &self,
        args: HttpRequestEndpointArgs,
    ) -> HttpRequestEndpointResult {
        self.test_env.call_canister_method_with_panic(
            self.canister_id,
            self.principal,
            "http_request_via_proxy",
            (args,),
        )
    }

    pub fn query_get_callback_responses(&self) -> HashMap<HttpRequestId, HttpResponse> {
        self.test_env.query_canister_method_with_panic(
            self.canister_id,
            self.principal,
            "get_callback_responses",
            (),
        )
    }
}

pub struct ProxyCanisterActor<'a> {
    test_env: &'a TestEnv,
    canister_id: Principal,
}

impl<'a> ProxyCanisterActor<'a> {
    pub fn new(test_env: &'a TestEnv, canister_id: Principal) -> Self {
        Self {
            test_env,
            canister_id,
        }
    }

    pub fn call_http_request(
        &self,
        args: HttpRequestEndpointArgs,
        caller: Principal,
    ) -> HttpRequestEndpointResult {
        self.test_env.call_canister_method_with_panic(
            self.canister_id,
            caller,
            "http_request_via_proxy",
            (args,),
        )
    }
}
