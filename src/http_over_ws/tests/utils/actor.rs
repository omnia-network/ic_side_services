use std::fmt;

use candid::Principal;
use http_over_ws::{HttpRequest, HttpRequestId};

use super::{ic_env::TestEnv, identity::generate_random_principal};

pub enum CanisterMethod {
    WsOpen,
    WsMessage,
    WsClose,
    WsGetMessages,
    ExecuteHttpRequest,
}

impl fmt::Display for CanisterMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CanisterMethod::WsOpen => write!(f, "ws_open"),
            CanisterMethod::WsMessage => write!(f, "ws_message"),
            CanisterMethod::WsClose => write!(f, "ws_close"),
            CanisterMethod::WsGetMessages => write!(f, "ws_get_messages"),
            CanisterMethod::ExecuteHttpRequest => write!(f, "execute_http_request"),
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

    pub fn call_execute_http_request(&self, args: HttpRequest) -> HttpRequestId {
        let res: HttpRequestId = self.test_env.call_canister_method_with_panic(
            self.principal,
            CanisterMethod::ExecuteHttpRequest,
            args,
        );

        res
    }
}
