mod utils;

use http_over_ws::{HttpMethod, HttpOverWsMessage, HttpRequest, HttpRequestFailureReason};
use utils::{actor::CanisterActor, constants::TEST_HEADER, ic_env, proxy_client::ProxyClient};

use crate::utils::constants::TEST_URL;

#[test]
fn test_execute_http_request() {
    let test_env = ic_env::get_test_env();
    let mut proxy_client = ProxyClient::new(&test_env);
    let canister_actor = CanisterActor::new(&test_env);

    proxy_client.setup_proxy();

    let request = HttpRequest::new(TEST_URL, HttpMethod::GET, vec![TEST_HEADER.clone()], None);

    let request_id = canister_actor.call_execute_http_request(request.clone());

    let proxy_messages = proxy_client.get_http_over_ws_messages();
    assert_eq!(
        proxy_messages[0],
        HttpOverWsMessage::HttpRequest(request_id, request),
    );

    let http_response = canister_actor.query_get_http_response(request_id);
    assert!(matches!(
        http_response,
        Err(HttpRequestFailureReason::Unknown)
    ));
}
