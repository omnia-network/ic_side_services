use std::cell::Cell;

use base64::{engine::general_purpose, Engine};
use ecdsa_api::{
    get_canister_ecdsa_public_key, set_canister_ecdsa_public_key, set_ecdsa_key_name,
    EcdsaPublicKey,
};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};

use flux::FluxNetwork;
use logger::log;
use ws::init_ws;

mod ecdsa_api;
mod flux;
mod flux_api;
mod http_over_ws;
mod logger;
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
    ic_cdk::storage::stable_save((network, ecdsa_pub_key))
        .expect("Saving network to stable store must succeed.");
}

#[post_upgrade]
fn post_upgrade() {
    let (network, ecdsa_pub_key) =
        ic_cdk::storage::stable_restore::<(FluxNetwork, EcdsaPublicKey)>()
            .expect("Failed to read network from stable memory.");

    init(network);

    set_canister_ecdsa_public_key(ecdsa_pub_key);
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
fn flux_login() {
    flux_api::authentication::login();
}

#[update]
fn flux_logout() {
    flux_api::authentication::logout();
}

#[update]
fn flux_fetch_balance() {
    flux_api::balance::fetch_balance();
}

#[query]
fn flux_get_balance() -> Option<i32> {
    flux_api::balance::get_balance()
}

#[query]
fn flux_is_logged_in() -> bool {
    flux_api::authentication::is_logged_in()
}
