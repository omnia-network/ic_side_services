use std::cell::Cell;

use base64::{engine::general_purpose, Engine};
use ecdsa_api::{
    get_canister_ecdsa_public_key, set_canister_ecdsa_public_key, set_ecdsa_key_name,
    EcdsaPublicKey,
};
use flux_api::authentication::{get_zelidauth, set_zelidauth};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};

use flux::FluxNetwork;
use logger::log;
use ws::init_ws;

mod ecdsa_api;
mod flux;
mod flux_api;
mod http_over_ws;
mod logger;
mod utils;
mod ws;

thread_local! {
    /// The Flux network to connect to.
    /* stable */ static NETWORK: Cell<FluxNetwork> = Cell::new(FluxNetwork::Local);
}

#[init]
fn init(network: FluxNetwork) {
    init_ws();

    NETWORK.with(|n| n.set(network));

    let key_name = set_ecdsa_key_name(network);

    log(&format!(
        "ecdsa_config: key_name: {}, network: {:?}",
        key_name, network
    ));
}

#[pre_upgrade]
fn pre_upgrade() {
    let network = NETWORK.with(|n| n.get());
    let ecdsa_pub_key = get_canister_ecdsa_public_key();
    let zelidauth = get_zelidauth().map(|h| h.value);

    ic_cdk::storage::stable_save((network, ecdsa_pub_key, zelidauth))
        .expect("Saving network to stable store must succeed.");
}

#[post_upgrade]
fn post_upgrade() {
    let (network, ecdsa_pub_key, zelidauth) =
        ic_cdk::storage::stable_restore::<(FluxNetwork, EcdsaPublicKey, Option<String>)>()
            .expect("Failed to read network from stable memory.");

    init(network);
    set_canister_ecdsa_public_key(ecdsa_pub_key);
    set_zelidauth(zelidauth);
}

/// Sets the ECDSA public key by fetching it from the ECDSA API.
#[update]
async fn set_canister_public_key(derivation_path: Option<String>) {
    let derivation_path = derivation_path
        .map(|dp| vec![dp.into_bytes()])
        .unwrap_or(vec![]);
    ecdsa_api::fetch_canister_ecdsa_public_key(derivation_path).await;
}

#[query]
fn get_addresses() -> (String, String) {
    let network = NETWORK.with(|n| n.get());
    (
        flux::get_p2pkh_address(network, flux::P2PKHAddress::ZCash),
        flux::get_p2pkh_address(network, flux::P2PKHAddress::ZelId),
    )
}

/// Signs a message with ECDSA and returns the base64-encoded signature.
#[update]
async fn sign_with_ecdsa(message: String, derivation_path: Option<String>) -> String {
    let derivation_path = derivation_path
        .map(|dp| vec![dp.into_bytes()])
        .unwrap_or(vec![]);
    let message_hash = flux::get_message_magic_hash(message);
    let signature = ecdsa_api::sign_with_ecdsa(derivation_path, message_hash).await;
    let signature_bytes = flux::encode_signature(&signature);
    general_purpose::STANDARD.encode(signature_bytes)
}

#[update]
fn flux_login() -> http_over_ws::HttpRequestId {
    flux_api::authentication::login()
}

#[update]
fn flux_logout() -> http_over_ws::HttpRequestId {
    flux_api::authentication::logout()
}

#[update]
fn flux_fetch_balance() -> http_over_ws::HttpRequestId {
    flux_api::balance::fetch_balance()
}

#[query]
fn flux_get_balance() -> Option<f32> {
    flux_api::balance::get_balance()
}

#[query]
fn flux_is_logged_in() -> bool {
    flux_api::authentication::is_logged_in()
}

/// Temporary deployment info.
/// It'll be the input of [flux_calculate_app_price] and [flux_register_app] methods.
fn tmp_deployment_info() -> flux_api::deployment::DeploymentInfo {
    let mut compose = flux_api::deployment::ComposeSpec::new();
    compose.name = Some("ichttprequestexecutor".to_string());
    compose.description = Some("IC HTTP Request Executor client".to_string());
    compose.repotag = Some("omniadevs/ic-http-request-executor:v0.0.4".to_string());
    compose.ports = Some(vec![80]);
    compose.domains = Some(vec![String::new()]);
    compose.environment_parameters = Some(vec![
        "IC_NETWORK_URL=https://icp0.io".to_string(),
        "IC_WS_GATEWAY_URL=wss://gateway.icws.io".to_string(),
        "CANISTER_ID_IC_SIDE_SERVICES_BACKEND=5fhww-dyaaa-aaaao-a26ia-cai".to_string(),
    ]);
    compose.commands = Some(vec![]);
    compose.container_ports = Some(vec![80]);
    compose.container_data = Some("/data".to_string());
    compose.cpu = Some(0.1); // (cores) min: 0.1 max: 15.0
    compose.ram = Some(100); // (MB) min: 100 max: 59000
    compose.hdd = Some(1); // (GB) min: 1 max: 840
    compose.tiered = Some(false);
    compose.secrets = Some(String::new()); // must be included as empty string
    compose.repoauth = Some(String::new()); // must be included as empty string

    flux_api::deployment::DeploymentInfo {
        compose,
        instances: 3,
        expires_after_blocks: 5000,
        static_ip: false,
    }
}

#[update]
fn flux_calculate_app_price() -> http_over_ws::HttpRequestId {
    flux_api::deployment::calculate_app_price(tmp_deployment_info())
}

#[update]
async fn flux_register_app() -> http_over_ws::HttpRequestId {
    flux_api::deployment::register_app(tmp_deployment_info()).await
}

#[update]
fn flux_get_deployment_information() -> http_over_ws::HttpRequestId {
    flux_api::deployment::fetch_deployment_information()
}
