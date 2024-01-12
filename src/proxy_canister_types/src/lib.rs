use candid::{CandidType, Deserialize, Principal};
use http_over_ws::{HttpRequest, HttpRequestId, HttpRequestTimeoutMs};

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
