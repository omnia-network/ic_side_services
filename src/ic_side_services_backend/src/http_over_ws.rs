use std::{cell::RefCell, collections::HashMap, ops::Deref};

use candid::Principal;
use flux_types::models::{
    verify_login_200_response, Appregister200Response, GetAppPrice200Response,
    LoginPhrase200Response, Status, VerifyLogin200Response, ZelIdLogin,
};
use ic_cdk::{trap, update};
use proxy_canister_types::{
    CanisterCallbackMethodName, HttpHeader, HttpMethod, HttpRequest, HttpRequestEndpointArgs,
    HttpRequestEndpointResult, HttpRequestId, HttpResult,
};
use url::Url;

use crate::{
    flux,
    flux_api::{
        deployment::DeploymentInformationResponse, CONTENT_TYPE_TEXT_PLAIN_HEADER,
        DEFAULT_HTTP_REQUEST_TIMEOUT_MS, FLUX_API_BASE_URL, FLUX_STATE,
    },
    logger::log,
    sign_with_ecdsa, NETWORK,
};

thread_local! {
    /* flexible */ static PROXY_CANISTER_ID: RefCell<Principal> = RefCell::new(Principal::from_text("iustv-tiaaa-aaaao-a3aga-cai").unwrap());
    /* flexible */ static CALLBACK_RESULTS: RefCell<HashMap<HttpRequestId, HttpResult>> = RefCell::new(HashMap::new());
}

pub async fn execute_http_request(
    url: Url,
    method: HttpMethod,
    headers: Vec<HttpHeader>,
    body: Option<Vec<u8>>,
    callback_method_name: Option<CanisterCallbackMethodName>,
    timeout_ms: Option<u64>,
) -> HttpRequestEndpointResult {
    let proxy_canister_id = PROXY_CANISTER_ID.with(|p| p.borrow().clone());

    let request = HttpRequest {
        url: url.to_string(),
        method,
        headers,
        body,
    };

    let res: Result<(HttpRequestEndpointResult,), _> = ic_cdk::call(
        proxy_canister_id,
        "http_request",
        (HttpRequestEndpointArgs {
            request,
            timeout_ms,
            callback_method_name,
        },),
    )
    .await;

    match res {
        Ok(http_res) => http_res.0,
        Err(e) => {
            trap(format!("{:?}", e).as_str());
        }
    }
}

#[update]
async fn login_phrase_callback(request_id: HttpRequestId, res: HttpResult) {
    match res {
        HttpResult::Success(res) => {
            if res.status != 200 {
                log(&format!("loginphrase failed with status: {}", res.status));
                return;
            }

            let LoginPhrase200Response { data, status } =
                serde_json::from_slice(&res.body).unwrap();
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
                Some(serde_json::to_vec(&body).unwrap()),
                Some(String::from("verify_login_callback")),
                Some(DEFAULT_HTTP_REQUEST_TIMEOUT_MS),
            )
            .await;
        }
        HttpResult::Failure(reason) => {
            // TODO: handle failure
        }
    }
}

#[update]
async fn verify_login_callback(request_id: HttpRequestId, res: HttpResult) {
    match res {
        HttpResult::Success(res) => {
            if res.status != 200 {
                log(&format!("verifylogin failed with status: {}", res.status));
                return;
            }

            let VerifyLogin200Response { data, status } =
                serde_json::from_slice(&res.body).unwrap();
            if let verify_login_200_response::Status::Error = status.unwrap() {
                log(&format!("verifylogin error: {:?}", data));
                return;
            }

            FLUX_STATE.with(|h| {
                h.borrow_mut()
                    .set_auth_header_from_verifylogin_response_data(*data.unwrap());
            });
        }
        HttpResult::Failure(reason) => {
            // TODO: handle failure
        }
    }
}

#[update]
async fn logout_callback(request_id: HttpRequestId, res: HttpResult) {
    match res {
        HttpResult::Success(res) => {
            if res.status != 200 {
                log(&format!("logout failed with status: {}", res.status));
                return;
            }

            log("logout successful");

            FLUX_STATE.with(|b| b.borrow_mut().reset_auth_header());
        }
        HttpResult::Failure(reason) => {
            // TODO: handle failure
        }
    }
}

#[update]
async fn balance_callback(request_id: HttpRequestId, res: HttpResult) {
    match res {
        HttpResult::Success(res) => {
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
        HttpResult::Failure(reason) => {
            // TODO: handle failure
        }
    }
}

#[update]
async fn calculate_price_callback(request_id: HttpRequestId, res: HttpResult) {
    match res {
        HttpResult::Success(res) => {
            if res.status != 200 {
                log(&format!(
                    "calculateappprice failed with status: {}",
                    res.status
                ));
                return;
            }

            let GetAppPrice200Response { status, data } =
                serde_json::from_slice(&res.body).unwrap();
            if let Status::Error = status.unwrap() {
                log(&format!("calculateappprice error: {:?}", data));
                return;
            }

            log(&format!("calculateappprice response: {:?}", data));
        }
        HttpResult::Failure(reason) => {
            // TODO: handle failure
        }
    }
}

#[update]
async fn app_register_callback(request_id: HttpRequestId, res: HttpResult) {
    match res {
        HttpResult::Success(res) => {
            if res.status != 200 {
                log(&format!("appregister failed with status: {}", res.status));
                return;
            }

            let Appregister200Response { status, data } =
                serde_json::from_slice(&res.body).unwrap();
            if let Status::Error = status.unwrap() {
                log(&format!("appregister error: {:?}", data));
                return;
            }

            log(&format!("appregister response: {:?}", data));
        }
        HttpResult::Failure(reason) => {
            // TODO: handle failure
        }
    }
}

#[update]
async fn deployment_information_callback(request_id: HttpRequestId, res: HttpResult) {
    match res {
        HttpResult::Success(res) => {
            if res.status != 200 {
                log(&format!(
                    "deploymentinformation failed with status: {}",
                    res.status
                ));
                return;
            }

            let DeploymentInformationResponse { status, data } =
                serde_json::from_slice(&res.body).unwrap();
            if let Status::Error = status.unwrap() {
                log(&format!("deploymentinformation error: {:?}", data));
                return;
            }

            log(&format!(
                "deploymentinformation address: {:?}",
                data.unwrap().address
            ));
        }
        HttpResult::Failure(reason) => {
            // TODO: handle failure
        }
    }
}
