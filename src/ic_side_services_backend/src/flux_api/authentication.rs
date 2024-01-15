use crate::{
    flux_api::{
        CONTENT_TYPE_TEXT_PLAIN_HEADER, DEFAULT_HTTP_REQUEST_TIMEOUT_MS, FLUX_API_BASE_URL,
        FLUX_STATE,
    },
    http_over_ws::execute_http_request,
};
use proxy_canister_types::{HttpHeader, HttpMethod, HttpRequestEndpointResult};
use std::ops::Deref;

pub async fn login() -> HttpRequestEndpointResult {
    let loginphrase_url = FLUX_API_BASE_URL.join("/id/loginphrase").unwrap();

    execute_http_request(
        loginphrase_url,
        HttpMethod::GET,
        vec![CONTENT_TYPE_TEXT_PLAIN_HEADER.deref().clone()],
        None,
        Some(String::from("login_phrase_callback")),
        // this request can take longer to complete due to the sign_with_ecdsa in the callback
        Some(2 * DEFAULT_HTTP_REQUEST_TIMEOUT_MS),
    )
    .await
}

pub async fn logout() -> HttpRequestEndpointResult {
    let zelidauth = get_zelidauth_or_trap();
    let logout_url = FLUX_API_BASE_URL.join("/id/logoutcurrentsession").unwrap();

    execute_http_request(
        logout_url,
        HttpMethod::GET,
        vec![zelidauth],
        None,
        Some(String::from("logout_callback")),
        Some(DEFAULT_HTTP_REQUEST_TIMEOUT_MS),
    )
    .await
}

pub fn get_zelidauth() -> Option<HttpHeader> {
    FLUX_STATE.with(|b| b.borrow().get_zelid_auth_header())
}

pub fn get_zelidauth_or_trap() -> HttpHeader {
    FLUX_STATE.with(|b| b.borrow().get_zelid_auth_header_or_trap())
}

pub fn set_zelidauth(zelidauth: Option<String>) {
    if let Some(zelidauth) = zelidauth {
        FLUX_STATE.with(|b| b.borrow_mut().set_auth_header(zelidauth));
    }
}

pub fn is_logged_in() -> bool {
    FLUX_STATE.with(|b| b.borrow().get_zelid_auth_header().is_some())
}
