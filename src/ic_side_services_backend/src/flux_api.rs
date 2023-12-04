use std::{cell::RefCell, ops::Deref};

use lazy_static::lazy_static;
use url::Url;

use crate::{
    flux,
    http_over_ws::{execute_http_request, HttpHeader, HttpMethod, HttpResponse},
    logger::log,
    sign_with_ecdsa, NETWORK,
};

lazy_static! {
    static ref FLUX_API_BASE_URL: Url = Url::parse("https://api.runonflux.io").unwrap();
    static ref CONTENT_TYPE_TEXT_PLAIN_HEADER: HttpHeader = HttpHeader {
        name: String::from("Content-Type"),
        value: String::from("text/plain"),
    };
}

thread_local! {
    /* flexible */ static ZELIDAUTH_HEADER: RefCell<Option<String>> = RefCell::new(None)
}

pub fn login() {
    let loginphrase_url = FLUX_API_BASE_URL.join("/id/loginphrase").unwrap();

    async fn verifylogin_cb(res: HttpResponse) {
        if res.status != 200 {
            log(&format!("verifylogin failed with status: {}", res.status));
            return;
        }

        let flux_types::models::VerifyLogin200Response { data, .. } =
            serde_json::from_slice(&res.body).unwrap();

        log(&format!("verifylogin response: {:?}", data));

        let response_data = data.unwrap();
        let zelid = response_data.zelid.unwrap();
        let login_phrase = response_data.login_phrase.unwrap();
        let signature = response_data.signature.unwrap();

        // workaround to get URL-encoded zelidauth
        let zelid_auth_header = Url::parse_with_params(
            "http://dummy.com",
            &[
                ("zelid", &zelid),
                ("signature", &signature),
                ("loginPhrase", &login_phrase),
            ],
        )
        .unwrap()
        .to_string()
        .split("?")
        .nth(1)
        .unwrap()
        .to_string();

        ZELIDAUTH_HEADER.with(|h| {
            *h.borrow_mut() = Some(zelid_auth_header.clone());
        });

        log(&format!("zelid auth header: {}", zelid_auth_header));
    }

    async fn loginphrase_cb(res: HttpResponse) {
        if res.status != 200 {
            log(&format!("loginphrase failed with status: {}", res.status));
            return;
        }

        let flux_types::models::LoginPhrase200Response { data, .. } =
            serde_json::from_slice(&res.body).unwrap();

        let login_phrase = data.unwrap();

        log(&format!("loginphrase: {}", login_phrase));

        // get the signature for the loginphrase
        let signature = sign_with_ecdsa(login_phrase.clone(), None).await;

        let body = flux_types::models::ZelIdLogin {
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
        );
    }

    execute_http_request(
        loginphrase_url,
        HttpMethod::GET,
        vec![CONTENT_TYPE_TEXT_PLAIN_HEADER.deref().clone()],
        None,
        Some(|res| Box::pin(loginphrase_cb(res))),
    );
}
