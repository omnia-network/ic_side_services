use std::{cell::RefCell, ops::Deref};

use ic_cdk::trap;
use lazy_static::lazy_static;
use url::Url;

use crate::{
    flux,
    http_over_ws::{execute_http_request, HttpHeader, HttpMethod, HttpResponse},
    logger::log,
    sign_with_ecdsa, NETWORK,
};

const DEFAULT_HTTP_REQUEST_TIMEOUT_MS: u64 = 15_000;

const ZELIDAUTH_HEADER_NAME: &str = "zelidauth";

lazy_static! {
    static ref FLUX_API_BASE_URL: Url = Url::parse("https://api.runonflux.io").unwrap();
    static ref CONTENT_TYPE_TEXT_PLAIN_HEADER: HttpHeader = HttpHeader {
        name: String::from("Content-Type"),
        value: String::from("text/plain"),
    };
}

thread_local! {
    /* flexible */ static FLUX_STATE: RefCell<FluxState> = RefCell::default();
}

#[derive(Default)]
struct FluxState {
    pub zelid_auth_header: Option<HttpHeader>,
    pub flux_balance: Option<i32>,
}

impl FluxState {
    fn set_auth_header(&mut self, zelid_auth: String) {
        self.zelid_auth_header = Some(HttpHeader {
            name: String::from(ZELIDAUTH_HEADER_NAME),
            value: zelid_auth.clone(),
        });

        log(&format!("set zelid auth header: {}", zelid_auth));
    }

    fn set_auth_header_from_verifylogin_response(
        &mut self,
        res: &flux_types::models::VerifyLogin200Response,
    ) {
        let data = res.data.to_owned().unwrap().to_owned();
        log(&format!("verifylogin response: {:?}", data));

        let zelid = data.zelid.unwrap();
        let login_phrase = data.login_phrase.unwrap();
        let signature = data.signature.unwrap();

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

        self.set_auth_header(zelid_auth_header);
    }

    fn get_zelid_auth_header_or_trap(&self) -> HttpHeader {
        if self.zelid_auth_header.is_none() {
            trap("zelid auth header not set. call login() first");
        }

        self.zelid_auth_header.to_owned().unwrap()
    }

    fn reset_auth_header(&mut self) {
        self.zelid_auth_header = None;
    }

    fn set_balance(&mut self, flux_balance: i32) {
        self.flux_balance = Some(flux_balance);

        log(&format!("set flux balance: {}", flux_balance));
    }

    fn set_balance_from_getaddressbalance_response(
        &mut self,
        res: &flux_types::models::get_address_balance_200_response::GetAddressBalance200Response,
    ) {
        let balance = res.data.unwrap();
        self.set_balance(balance);
    }

    fn get_balance(&self) -> Option<i32> {
        self.flux_balance
    }

    fn check_balance(&self) -> bool {
        self.flux_balance.is_some_and(|balance| balance > 0)
    }
}

pub fn login() {
    let loginphrase_url = FLUX_API_BASE_URL.join("/id/loginphrase").unwrap();

    async fn verifylogin_cb(res: HttpResponse) {
        if res.status != 200 {
            log(&format!("verifylogin failed with status: {}", res.status));
            return;
        }

        let res_body = serde_json::from_slice(&res.body).unwrap();

        FLUX_STATE.with(|h| {
            h.borrow_mut()
                .set_auth_header_from_verifylogin_response(&res_body);
        });
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

pub fn fetch_balance() {
    let mut balance_url = FLUX_API_BASE_URL.join("/explorer/balance").unwrap();
    balance_url.query_pairs_mut().append_pair(
        "address",
        &flux::get_p2pkh_address(NETWORK.with(|n| n.get()), flux::P2PKHAddress::ZCash),
    );

    async fn balance_cb(res: HttpResponse) {
        if res.status != 200 {
            log(&format!("balance failed with status: {}", res.status));
            return;
        }

        let res_body = serde_json::from_slice(&res.body).unwrap();

        FLUX_STATE.with(|b| {
            b.borrow_mut()
                .set_balance_from_getaddressbalance_response(&res_body)
        });
    }

    execute_http_request(
        balance_url,
        HttpMethod::GET,
        vec![],
        None,
        Some(|res| Box::pin(balance_cb(res))),
        Some(DEFAULT_HTTP_REQUEST_TIMEOUT_MS),
    );
}

pub fn get_balance() -> Option<i32> {
    FLUX_STATE.with(|b| b.borrow().get_balance())
}
