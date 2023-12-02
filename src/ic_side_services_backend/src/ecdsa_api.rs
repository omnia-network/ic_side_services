use std::cell::RefCell;

use ic_cdk::{
    api::management_canister::ecdsa::{
        self, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument, SignWithEcdsaArgument,
    },
    trap,
};

use crate::{flux::FluxNetwork, logger::log};

thread_local! {
    // The ECDSA key name.
    /* flexible */ static KEY_NAME: RefCell<String> = RefCell::new(String::from(""));
    /// The ECDSA canister public key, obtained from [ecdsa_api::ecdsa_public_key].
    /* flexible */ static CANISTER_ECDSA_PUBLIC_KEY: RefCell<Option<Vec<u8>>> = RefCell::new(None);
}

pub fn set_ecdsa_key_name(network: FluxNetwork) -> String {
    let key_name = match network {
        // For local development, we use a special test key with dfx.
        FluxNetwork::Local => "dfx_test_key",
        // On the IC, we can use test or production keys.
        FluxNetwork::Testnet => "test_key_1",
        FluxNetwork::Mainnet => "key_1",
    }
    .to_string();

    KEY_NAME.with(|s| {
        s.replace(key_name.clone());
    });

    key_name
}

fn get_ecdsa_key_name() -> String {
    KEY_NAME.with(|s| s.borrow().clone())
}

/// Fetches the ECDSA public key at the given derivation path.
pub async fn set_canister_ecdsa_public_key(derivation_path: Vec<Vec<u8>>) {
    let key_name = get_ecdsa_key_name();
    // Fetch the public key of the given derivation path.
    let public_key = ecdsa_public_key(derivation_path.clone()).await;
    CANISTER_ECDSA_PUBLIC_KEY.with(|k| {
        k.replace(Some(public_key.clone()));
    });

    log(&format!(
        "set_canister_ecdsa_public_key: key_name: {}, derivation_path: {:?}, public_key: {:?}",
        key_name, derivation_path, public_key
    ));
}

/// Returns the ECDSA public key obtained from [set_canister_ecdsa_public_key].
pub fn get_canister_ecdsa_public_key() -> Vec<u8> {
    match CANISTER_ECDSA_PUBLIC_KEY.with(|k| k.borrow().clone()) {
        Some(pk) => pk,
        None => {
            trap(
                "Canister ECDSA public key is not set. Call the set_canister_public_key method first.",
            );
        }
    }
}

/// Returns the ECDSA public key of this canister at the given derivation path.
pub async fn ecdsa_public_key(derivation_path: Vec<Vec<u8>>) -> Vec<u8> {
    let key_name = get_ecdsa_key_name();
    // Retrieve the public key of this canister at the given derivation path
    // from the ECDSA API.
    let res = ecdsa::ecdsa_public_key(EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path,
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: key_name,
        },
    })
    .await;

    res.unwrap().0.public_key
}

pub async fn sign_with_ecdsa(derivation_path: Vec<Vec<u8>>, message_hash: Vec<u8>) -> Vec<u8> {
    let key_name = get_ecdsa_key_name();
    let res = ecdsa::sign_with_ecdsa(SignWithEcdsaArgument {
        message_hash,
        derivation_path,
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: key_name.clone(),
        },
    })
    .await;

    res.unwrap().0.signature
}
