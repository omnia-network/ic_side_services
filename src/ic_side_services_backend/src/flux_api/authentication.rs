use std::ops::Deref;

use flux_types::models::*;

use crate::{
    flux,
    flux_api::{
        CONTENT_TYPE_TEXT_PLAIN_HEADER, DEFAULT_HTTP_REQUEST_TIMEOUT_MS, FLUX_API_BASE_URL,
        FLUX_STATE,
    },
    http_over_ws::{execute_http_request, HttpMethod, HttpResponse},
    logger::log,
    sign_with_ecdsa, NETWORK,
};

pub fn login() {
    let loginphrase_url = FLUX_API_BASE_URL.join("/id/loginphrase").unwrap();

    async fn verifylogin_cb(res: HttpResponse) {
        if res.status != 200 {
            log(&format!("verifylogin failed with status: {}", res.status));
            return;
        }

        let VerifyLogin200Response { data, status } = serde_json::from_slice(&res.body).unwrap();
        if let verify_login_200_response::Status::Error = status.unwrap() {
            log(&format!("verifylogin error: {:?}", data));
            return;
        }

        FLUX_STATE.with(|h| {
            h.borrow_mut()
                .set_auth_header_from_verifylogin_response_data(*data.unwrap());
        });
    }

    async fn loginphrase_cb(res: HttpResponse) {
        if res.status != 200 {
            log(&format!("loginphrase failed with status: {}", res.status));
            return;
        }

        let LoginPhrase200Response { data, status } = serde_json::from_slice(&res.body).unwrap();
        if let Status::Error = status.unwrap() {
            log(&format!("loginphrase error: {:?}", data));
            return;
        }

        let login_phrase = data.unwrap();

        log(&format!("loginphrase: {}", login_phrase));

        // get the signature for the loginphrase
        let signature = sign_with_ecdsa(login_phrase.clone(), None).await;

        let body = ZelIdLogin {
            login_phrase: Some(login_phrase),
            zelid: Some(flux::get_p2pkh_address(
                NETWORK.with(|n| n.get()),
                flux::P2PKHAddress::ZelId,
            )),
            signature: Some(signature),
        };

        let verifylogin_url = FLUX_API_BASE_URL.join("/id/verifylogin").unwrap();

        execute_http_request(
            verifylogin_url,
            HttpMethod::POST,
            vec![CONTENT_TYPE_TEXT_PLAIN_HEADER.deref().clone()],
            Some(serde_json::to_string(&body).unwrap()),
            Some(|res| Box::pin(verifylogin_cb(res))),
            Some(DEFAULT_HTTP_REQUEST_TIMEOUT_MS),
        );
    }

    execute_http_request(
        loginphrase_url,
        HttpMethod::GET,
        vec![CONTENT_TYPE_TEXT_PLAIN_HEADER.deref().clone()],
        None,
        Some(|res| Box::pin(loginphrase_cb(res))),
        // this request can take longer to complete due to the sign_with_ecdsa in the callback
        Some(2 * DEFAULT_HTTP_REQUEST_TIMEOUT_MS),
    );
}

pub fn logout() {
    let zelidauth = FLUX_STATE.with(|b| b.borrow().get_zelid_auth_header_or_trap());
    let logout_url = FLUX_API_BASE_URL.join("/id/logoutcurrentsession").unwrap();

    async fn logout_cb(res: HttpResponse) {
        if res.status != 200 {
            log(&format!("logout failed with status: {}", res.status));
            return;
        }

        log("logout successful");

        FLUX_STATE.with(|b| b.borrow_mut().reset_auth_header());
    }

    execute_http_request(
        logout_url,
        HttpMethod::GET,
        vec![zelidauth],
        None,
        Some(|res| Box::pin(logout_cb(res))),
        Some(DEFAULT_HTTP_REQUEST_TIMEOUT_MS),
    );
}

pub fn is_logged_in() -> bool {
    FLUX_STATE.with(|b| b.borrow().get_zelid_auth_header().is_some())
}
