use proxy_canister_types::{
    CanisterCallbackMethodName, CanisterId, HttpRequestEndpointArgs, InvalidRequest,
};
use url::Url;

use crate::constants::{
    MAX_HTTP_HEADERS_COUNT, MAX_HTTP_REQUEST_TIMEOUT_MS, MIN_HTTP_REQUEST_TIMEOUT_MS,
};

#[derive(Clone)]
pub enum RequestState {
    Executing(CanisterCallbackMethodName),
    Completed,
    Failed(String),
}

#[derive(Clone)]
pub struct CanisterRequest {
    pub canister_id: CanisterId,
    pub state: RequestState,
}

impl CanisterRequest {
    pub fn new(
        canister_id: CanisterId,
        callback_method_name: Option<CanisterCallbackMethodName>,
    ) -> Self {
        Self {
            canister_id,
            state: match callback_method_name {
                Some(name) => RequestState::Executing(name),
                None => RequestState::Completed,
            },
        }
    }

    pub fn complete(&mut self) {
        self.state = RequestState::Completed;
    }

    pub fn fail(&mut self, reason: String) {
        self.state = RequestState::Failed(reason);
    }
}

pub fn validate_incoming_request(args: &HttpRequestEndpointArgs) -> Result<(), InvalidRequest> {
    Url::parse(&args.request.url).map_err(|e| InvalidRequest::InvalidUrl(e.to_string()))?;

    if args.request.headers.len() > MAX_HTTP_HEADERS_COUNT {
        return Err(InvalidRequest::TooManyHeaders);
    }

    if args.timeout_ms.is_some_and(|timeout_ms| {
        timeout_ms > MAX_HTTP_REQUEST_TIMEOUT_MS || timeout_ms < MIN_HTTP_REQUEST_TIMEOUT_MS
    }) {
        return Err(InvalidRequest::InvalidTimeout);
    }

    Ok(())
}
