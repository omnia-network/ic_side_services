use crate::{
    flux,
    flux_api::{DEFAULT_HTTP_REQUEST_TIMEOUT_MS, FLUX_API_BASE_URL, FLUX_STATE},
    NETWORK,
};

use http_over_ws::{execute_http_request, HttpMethod, HttpRequestId, HttpResponse, HttpOverWsError};
use logger::log;

pub fn fetch_balance() -> Result<HttpRequestId, HttpOverWsError> {
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
        ic_websocket_cdk::send
    )
}

/// Returns FLUX token balance.
pub fn get_balance() -> Option<f32> {
    FLUX_STATE.with(|b| b.borrow().get_balance().map(|v| (v as f32) / 100_000_000.0))
}
