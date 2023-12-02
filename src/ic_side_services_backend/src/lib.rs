use std::cell::{Cell, RefCell};

use base64::{engine::general_purpose, Engine};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};

use flux::FluxNetwork;
use logger::log;
use ws::init_ws;

mod ecdsa_api;
mod flux;
mod http_over_ws;
mod logger;
mod ws;

thread_local! {
    // The bitcoin network to connect to.
    //
    // When developing locally this should be `Local`.
    // When deploying to the IC this should be `Testnet` or `Mainnet`.
    /* stable */ static NETWORK: Cell<FluxNetwork> = Cell::new(FluxNetwork::Local);

    // The ECDSA key name.
    /* flexible */ static KEY_NAME: RefCell<String> = RefCell::new(String::from(""));
}

#[init]
pub fn init(network: FluxNetwork) {
    init_ws();

    NETWORK.with(|n| n.set(network));

    let key_name = match network {
        // For local development, we use a special test key with dfx.
        FluxNetwork::Local => "dfx_test_key",
        // On the IC, we can use test or production keys.
        FluxNetwork::Testnet => "test_key_1",
        FluxNetwork::Mainnet => "key_1",
    };

    KEY_NAME.with(|s| {
        s.replace(String::from(key_name));
    });

    log(&format!(
        "get_ecdsa_config: key_name: {}, network: {:?}",
        key_name, network
    ));
}

/// Returns the P2PKH address of this canister at a specific derivation path.
#[update]
pub async fn set_canister_public_key(derivation_path: Option<String>) {
    let derivation_path = derivation_path
        .map(|dp| vec![dp.into_bytes()])
        .unwrap_or(vec![]);
    let key_name = KEY_NAME.with(|kn| kn.borrow().clone());
    flux::set_canister_ecdsa_public_key(key_name, derivation_path).await
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
pub async fn sign_with_ecdsa(message: String, derivation_path: Option<String>) -> String {
    let derivation_path = derivation_path
        .map(|dp| vec![dp.into_bytes()])
        .unwrap_or(vec![]);
    let key_name = KEY_NAME.with(|kn| kn.borrow().clone());
    let message_hash = flux::get_message_magic_hash(message);
    let signature = ecdsa_api::sign_with_ecdsa(key_name, derivation_path, message_hash).await;
    let signature_bytes = flux::encode_signature(&signature);
    general_purpose::STANDARD.encode(signature_bytes)
}

#[pre_upgrade]
fn pre_upgrade() {
    let network = NETWORK.with(|n| n.get());
    ic_cdk::storage::stable_save((network,)).expect("Saving network to stable store must succeed.");
}

#[post_upgrade]
fn post_upgrade() {
    let network = ic_cdk::storage::stable_restore::<(FluxNetwork,)>()
        .expect("Failed to read network from stable memory.")
        .0;

    init(network);
}
