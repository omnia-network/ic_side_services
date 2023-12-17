use std::ops::Deref;

use flux_types::models::*;

use crate::{
    flux,
    flux_api::{
        CONTENT_TYPE_TEXT_PLAIN_HEADER, DEFAULT_HTTP_REQUEST_TIMEOUT_MS, FLUX_API_BASE_URL,
    },
    http_over_ws::{execute_http_request, HttpMethod, HttpRequestId, HttpResponse},
    logger::log,
    NETWORK,
};

pub fn calculate_app_price(compose: GetAppPriceRequestComposeInner) -> HttpRequestId {
    let calculateappprice_url = FLUX_API_BASE_URL.join("/apps/calculateprice").unwrap();

    let body = GetAppPriceRequest {
        version: Some(7),
        name: compose.clone().name,
        description: compose.clone().description,
        owner: Some(flux::get_p2pkh_address(
            NETWORK.with(|n| n.get()),
            flux::P2PKHAddress::ZelId,
        )),
        compose: Some(vec![compose]),
        instances: Some(3),
        contacts: Some(vec![]),
        geolocation: Some(vec![]),
        expire: Some(5000), // 5000 blocks are ~ 1 week (according to Flux API)
        nodes: Some(vec![]),
        staticip: Some(false),
    };

    async fn calculateappprice_cb(res: HttpResponse) {
        if res.status != 200 {
            log(&format!(
                "calculateappprice failed with status: {}",
                res.status
            ));
            return;
        }

        let GetAppPrice200Response { status, data } = serde_json::from_slice(&res.body).unwrap();
        if let Status::Error = status.unwrap() {
            log(&format!("calculateappprice error: {:?}", data));
            return;
        }

        log(&format!("calculateappprice response: {:?}", data));
    }

    execute_http_request(
        calculateappprice_url,
        HttpMethod::POST,
        vec![CONTENT_TYPE_TEXT_PLAIN_HEADER.deref().clone()],
        Some(serde_json::to_string(&body).unwrap()),
        Some(|res| Box::pin(calculateappprice_cb(res))),
        // this request can take longer to complete due to the sign_with_ecdsa in the callback
        Some(2 * DEFAULT_HTTP_REQUEST_TIMEOUT_MS),
    )
}
