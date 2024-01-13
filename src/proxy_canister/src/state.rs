use std::collections::HashMap;

use http_over_ws::HttpConnectionId;
use proxy_canister_types::{CanisterCallbackMethodName, CanisterId, CanisterRequest};

pub struct ProxyState {
    requests: HashMap<HttpConnectionId, CanisterRequest>,
}

impl ProxyState {
    pub fn new() -> Self {
        Self {
            requests: HashMap::new(),
        }
    }

    pub fn start_request_for_canister(
        &mut self,
        canister_id: CanisterId,
        request_id: HttpConnectionId,
        callback_method_name: Option<CanisterCallbackMethodName>,
    ) {
        self.requests
            .entry(request_id)
            .or_insert(CanisterRequest::new(canister_id, callback_method_name));
    }

    pub fn get_request_state(&self, request_id: HttpConnectionId) -> Option<CanisterRequest> {
        self.requests.get(&request_id).cloned()
    }

    pub fn set_request_successful(&mut self, request_id: HttpConnectionId) {
        self.requests.entry(request_id).and_modify(|r| r.success());
    }

    pub fn set_request_failed(&mut self, request_id: HttpConnectionId, reason: String) {
        self.requests
            .entry(request_id)
            .and_modify(|r| r.fail(reason));
    }
}
