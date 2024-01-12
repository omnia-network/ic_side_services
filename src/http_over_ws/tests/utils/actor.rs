use std::fmt;

use candid::Principal;
use http_over_ws::{
    ExecuteHttpRequestResult, GetHttpResponseResult, HttpRequest, HttpConnectionId,
    HttpRequestTimeoutMs, HttpResponse,
};

use super::{ic_env::TestEnv, identity::generate_random_principal};

pub enum CanisterMethod {
    WsOpen,
    WsMessage,
    WsClose,
    WsGetMessages,
    ExecuteHttpRequest,
    GetHttpResponse,
    GetCallbackResponses,
}

impl fmt::Display for CanisterMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CanisterMethod::WsOpen => write!(f, "ws_open"),
            CanisterMethod::WsMessage => write!(f, "ws_message"),
            CanisterMethod::WsClose => write!(f, "ws_close"),
            CanisterMethod::WsGetMessages => write!(f, "ws_get_messages"),
            CanisterMethod::ExecuteHttpRequest => write!(f, "execute_http_request"),
            CanisterMethod::GetHttpResponse => write!(f, "get_http_response"),
            CanisterMethod::GetCallbackResponses => write!(f, "get_callback_responses"),
        }
    }
}

pub struct CanisterActor<'a> {
    test_env: &'a TestEnv,
    principal: Principal,
}

impl<'a> CanisterActor<'a> {
    pub fn new(test_env: &'a TestEnv) -> Self {
        Self {
            test_env,
            principal: generate_random_principal(),
        }
    }

    pub fn call_execute_http_request(
        &self,
        req: HttpRequest,
        timeout_ms: Option<HttpRequestTimeoutMs>,
        with_callback: bool,
    ) -> ExecuteHttpRequestResult {
        self.test_env.call_canister_method_with_panic(
            self.principal,
            CanisterMethod::ExecuteHttpRequest,
            (req, timeout_ms, with_callback),
        )
    }

    pub fn query_get_http_response(&self, request_id: HttpConnectionId) -> GetHttpResponseResult {
        self.test_env.query_canister_method_with_panic(
            self.principal,
            CanisterMethod::GetHttpResponse,
            (request_id,),
        )
    }

    pub fn query_get_callback_responses(&self) -> Vec<HttpResponse> {
        self.test_env.query_canister_method_with_panic(
            self.principal,
            CanisterMethod::GetCallbackResponses,
            (),
        )
    }
}
