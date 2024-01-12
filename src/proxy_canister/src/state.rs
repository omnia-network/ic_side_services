use std::collections::HashMap;

use http_over_ws::HttpRequestId;

use crate::{requests::CanisterRequest, types::*};

pub struct ProxyState {
    requests: HashMap<HttpRequestId, CanisterRequest>,
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
        request_id: HttpRequestId,
        callback_method_name: Option<CanisterCallbackMethodName>,
    ) {
        self.requests
            .entry(request_id)
            .or_insert(CanisterRequest::new(canister_id, callback_method_name));
    }

    pub fn get_request_state(&self, request_id: HttpRequestId) -> Option<CanisterRequest> {
        self.requests.get(&request_id).cloned()
    }

    pub fn set_request_completed(&mut self, request_id: HttpRequestId) {
        self.requests.entry(request_id).and_modify(|r| r.complete());
    }

    pub fn set_request_failed(&mut self, request_id: HttpRequestId, reason: String) {
        self.requests
            .entry(request_id)
            .and_modify(|r| r.fail(reason));
    }
}
