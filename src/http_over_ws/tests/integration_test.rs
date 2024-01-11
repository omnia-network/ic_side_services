mod utils;

use http_over_ws::{HttpMethod, HttpRequest};
use utils::{actor::CanisterActor, ic_env, ws_proxy::WsProxyClient};

#[test]
fn test_execute_http_request() {
    let test_env = ic_env::get_test_env();
    let mut ws_proxy = WsProxyClient::new(&test_env);
    let canister_actor = CanisterActor::new(&test_env);

    ws_proxy.open_ws_connection();

    canister_actor.call_execute_http_request(HttpRequest::new(
        "https://ic0.app",
        HttpMethod::GET,
        vec![],
        None,
    ));

    // TODO: complete after merge
}
