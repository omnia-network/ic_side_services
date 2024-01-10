use std::cell::RefCell;

use flux_types::models::*;
use ic_cdk::trap;
use lazy_static::lazy_static;
use url::Url;

use http_over_ws::HttpHeader;
use logger::log;

pub mod authentication;
pub mod balance;
pub mod deployment;

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

    fn set_auth_header_from_verifylogin_response_data(&mut self, data: VerifyLogin200ResponseData) {
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

    fn get_zelid_auth_header(&self) -> Option<HttpHeader> {
        self.zelid_auth_header.clone()
    }

    fn get_zelid_auth_header_or_trap(&self) -> HttpHeader {
        self.get_zelid_auth_header().unwrap_or_else(|| {
            trap("zelid auth header not set. call login() first");
        })
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
        res: &get_address_balance_200_response::GetAddressBalance200Response,
    ) {
        let balance = res.data.unwrap();
        self.set_balance(balance);
    }

    fn get_balance(&self) -> Option<i32> {
        self.flux_balance
    }

    // fn check_balance(&self) -> bool {
    //     self.flux_balance.is_some_and(|balance| balance > 0)
    // }
}
