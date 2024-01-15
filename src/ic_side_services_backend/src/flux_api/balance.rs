use crate::{
    flux,
    flux_api::{DEFAULT_HTTP_REQUEST_TIMEOUT_MS, FLUX_API_BASE_URL, FLUX_STATE},
    http_over_ws::execute_http_request,
    NETWORK,
};
use proxy_canister_types::{HttpMethod, HttpRequestEndpointResult};

pub async fn fetch_balance() -> HttpRequestEndpointResult {
    let mut balance_url = FLUX_API_BASE_URL.join("/explorer/balance").unwrap();
    balance_url.query_pairs_mut().append_pair(
        "address",
        &flux::get_p2pkh_address(NETWORK.with(|n| n.get()), flux::P2PKHAddress::ZCash),
    );

    execute_http_request(
        balance_url,
        HttpMethod::GET,
        vec![],
        None,
        Some(String::from("balance_callback")),
        Some(DEFAULT_HTTP_REQUEST_TIMEOUT_MS),
    )
    .await
}

/// Returns FLUX token balance.
pub fn get_balance() -> Option<f32> {
    FLUX_STATE.with(|b| b.borrow().get_balance().map(|v| (v as f32) / 100_000_000.0))
}
