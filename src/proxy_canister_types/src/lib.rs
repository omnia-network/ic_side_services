use candid::{CandidType, Deserialize, Principal};
use http_over_ws::{HttpRequestId, HttpRequest, HttpRequestTimeoutMs};

pub type CanisterId = Principal;
pub type CanisterCallbackMethodName = String;

#[derive(CandidType, Deserialize, Debug, PartialEq, Eq)]
pub struct HttpRequestEndpointArgs {
    pub request: HttpRequest,
    pub timeout_ms: Option<HttpRequestTimeoutMs>,
    pub callback_method_name: Option<CanisterCallbackMethodName>,
}

pub type HttpRequestEndpointResult = Result<HttpRequestId, ProxyError>;

#[derive(CandidType, Deserialize, Debug, PartialEq, Eq)]
pub enum ProxyError {
    InvalidRequest(InvalidRequest),
    Generic(String),
}

#[derive(CandidType, Deserialize, Debug, PartialEq, Eq)]
pub enum InvalidRequest {
    InvalidUrl(String),
    TooManyHeaders,
    InvalidTimeout,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RequestState {
    Executing(Option<CanisterCallbackMethodName>),
    Successful,
    Failed(String),
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
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
            state: RequestState::Executing(callback_method_name),
        }
    }

    pub fn success(&mut self) {
        self.state = RequestState::Successful;
    }

    pub fn fail(&mut self, reason: String) {
        self.state = RequestState::Failed(reason);
    }
}
