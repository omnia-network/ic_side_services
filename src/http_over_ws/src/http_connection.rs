use std::{future::Future, pin::Pin};

use candid::{decode_one, encode_one, CandidType, Deserialize};
use ic_cdk::api::management_canister::http_request::{
    HttpHeader as ApiHttpHeader, HttpResponse as ApiHttpResponse,
};
use ic_cdk_timers::TimerId;
use logger::log;

pub type HttpConnectionId = u64;

pub type ExecuteHttpRequestResult = Result<HttpConnectionId, HttpOverWsError>;
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
pub type HttpCallback = fn(HttpResponse) -> Pin<Box<dyn Future<Output = ()>>>;

pub type HttpRequestTimeoutMs = u64;

#[derive(CandidType, Debug, Deserialize, PartialEq, Eq)]
pub enum HttpOverWsMessage {
    SetupProxyClient,
    HttpRequest(HttpConnectionId, HttpRequest),
    HttpResponse(HttpConnectionId, HttpResponse),
    Error(Option<HttpConnectionId>, String),
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
pub(crate) struct HttpConnection {
    id: HttpConnectionId,
    request: HttpRequest,
    state: HttpConnectionState,
}

impl HttpConnection {
    pub fn new(
        id: HttpConnectionId,
        request: HttpRequest,
        callback: Option<HttpCallback>,
        timer_id: Option<TimerId>,
    ) -> Self {
        HttpConnection {
            id,
            request,
            state: HttpConnectionState::new(timer_id, callback),
        }
    }

    pub fn get_request(&self) -> HttpRequest {
        self.request.clone()
    }

    pub fn get_response(&self) -> GetHttpResponseResult {
        match self.state {
            HttpConnectionState::WaitingForResponse(_) => Err(HttpFailureReason::Unknown),
            HttpConnectionState::Failed(ref reason) => Err(reason.clone()),
            HttpConnectionState::Success(ref response) => Ok(response.clone()),
        }
    }

    pub fn update_state(&mut self, response: HttpResponse) -> Result<(), HttpFailureReason> {
        match &mut self.state {
            HttpConnectionState::WaitingForResponse((timer_id, callback)) => {
                // response has been received, clear the timer if it was set
                if let Some(timer_id) = timer_id.take() {
                    ic_cdk_timers::clear_timer(timer_id);
                }
        
                // if a callback was set, execute it
                if let Some(callback) = callback.take() {
                    ic_cdk::spawn(
                        {
                            let response = response.clone();
                            async move { callback(response).await }
                        });
                }
                self.state = HttpConnectionState::Success(response);
                Ok(())
            },
            HttpConnectionState::Failed(e) => {
                Err(e.to_owned())
            }
            HttpConnectionState::Success(_) => {
                // a second response is ignored
                Ok(())
            }
        }
    }

    pub fn set_timeout(&mut self) {
        match self.state {
            HttpConnectionState::WaitingForResponse(_) => {
                log(&format!(
                    "http_over_ws: HTTP connection with id {} timed out",
                    self.id
                ));

                self.state = HttpConnectionState::Failed(HttpFailureReason::RequestTimeout);
            }
            HttpConnectionState::Failed(_) => {
                log(&format!(
                    "http_over_ws: HTTP connection with id {} timed out after it had already failed",
                    self.id
                ));
            }
            HttpConnectionState::Success(_) => {
                log(&format!(
                    "http_over_ws: HTTP connection with id {} timed out after it had already succeeded",
                    self.id
                ));
            }
        }
    }

    pub fn report_failure(&mut self, reason: HttpFailureReason) {
        match self.state {
            HttpConnectionState::WaitingForResponse(_) => {
                log(&format!(
                    "http_over_ws: HTTP connection with id {} failed with reason {:?}",
                    self.id,
                    reason
                ));

                self.state = HttpConnectionState::Failed(reason);
            },
            HttpConnectionState::Failed(_) => {
                log(&format!(
                    "http_over_ws: HTTP connection with id {} failed after it had already failed",
                    self.id
                ));
            }
            HttpConnectionState::Success(_) => {
                log(&format!(
                    "http_over_ws: HTTP connection with id {} failed after it had already succeeded",
                    self.id
                ));
            }
        }
    }
}

#[derive(Clone)]
pub (crate) enum HttpConnectionState {
    WaitingForResponse((Option<TimerId>, Option<HttpCallback>)),
    Failed(HttpFailureReason),
    Success(HttpResponse),
}

impl HttpConnectionState {
    pub fn new(timer_id: Option<TimerId>, callback: Option<HttpCallback>) -> Self {
        HttpConnectionState::WaitingForResponse((timer_id, callback))
    }
}
