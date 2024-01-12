use std::{future::Future, pin::Pin};

use candid::{decode_one, encode_one, CandidType, Deserialize};
use ic_cdk::api::management_canister::http_request::{
    HttpHeader as ApiHttpHeader, HttpResponse as ApiHttpResponse,
};
use ic_cdk_timers::TimerId;

pub type HttpRequestId = u32;
pub type ExecuteHttpRequestResult = Result<HttpRequestId, HttpOverWsError>;
pub type GetHttpResponseResult = Result<HttpResponse, HttpFailureReason>;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    HEAD,
    DELETE,
}

pub type HttpHeader = ApiHttpHeader;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Vec<HttpHeader>,
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    pub fn new(
        url: &str,
        method: HttpMethod,
        headers: Vec<HttpHeader>,
        body: Option<Vec<u8>>,
    ) -> Self {
        HttpRequest {
            url: url.to_string(),
            method,
            headers,
            body,
        }
    }
}

pub type HttpResponse = ApiHttpResponse;
pub type HttpCallback = fn(HttpRequestId, HttpResponse) -> Pin<Box<dyn Future<Output = ()>>>;

pub type HttpRequestTimeoutMs = u64;

#[derive(CandidType, Debug, Deserialize, PartialEq, Eq)]
pub enum HttpOverWsMessage {
    SetupProxyClient,
    HttpRequest(HttpRequestId, HttpRequest),
    HttpResponse(HttpRequestId, HttpResponse),
    Error(Option<HttpRequestId>, String),
}

impl HttpOverWsMessage {
    pub fn to_bytes(&self) -> Vec<u8> {
        encode_one(self).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        decode_one(bytes).map_err(|e| e.to_string())
    }
}

#[derive(CandidType, Debug, Deserialize, PartialEq, Eq)]
pub enum HttpOverWsError {
    /// The message is not an HttpOverWsMessage, therefore the on_message callback given to the IC WS cdk
    /// should try to parse it as its own message type.
    NotHttpOverWsType(String),
    /// The message is an HttpOverWsMessage, however it is not what it is expected to be.
    InvalidHttpMessage(HttpFailureReason),
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum HttpFailureReason {
    RequestTimeout,
    ProxyError(String),
    /// Used when retrieving the request from the state
    /// and the request is not found.
    RequestIdNotFound,
    Unknown,
}

#[derive(Clone)]
pub(crate) struct HttpRequestState {
    pub request: HttpRequest,
    pub response: Option<HttpResponse>,
    pub callback: Option<HttpCallback>,
    pub timer_id: Option<TimerId>,
    pub failure_reason: Option<HttpFailureReason>,
}

impl HttpRequestState {
    pub fn new(
        request: HttpRequest,
        callback: Option<HttpCallback>,
        timer_id: Option<TimerId>,
    ) -> Self {
        HttpRequestState {
            request,
            response: None,
            callback,
            timer_id,
            failure_reason: None,
        }
    }
}
