use ic_cdk::api::management_canister::ecdsa::{self, SignWithEcdsaArgument, EcdsaKeyId, EcdsaCurve, EcdsaPublicKeyArgument};

/// Returns the ECDSA public key of this canister at the given derivation path.
pub async fn ecdsa_public_key(key_name: String, derivation_path: Vec<Vec<u8>>) -> Vec<u8> {
    // Retrieve the public key of this canister at the given derivation path
    // from the ECDSA API.
    let res = ecdsa::ecdsa_public_key(EcdsaPublicKeyArgument {
      canister_id: None,
      derivation_path,
      key_id: EcdsaKeyId {
          curve: EcdsaCurve::Secp256k1,
          name: key_name,
      },
    }).await;

    res.unwrap().0.public_key
}

pub async fn sign_with_ecdsa(
    key_name: String,
    derivation_path: Vec<Vec<u8>>,
    message_hash: Vec<u8>,
) -> Vec<u8> {
  let res = ecdsa::sign_with_ecdsa(SignWithEcdsaArgument {
    message_hash,
    derivation_path,
    key_id: EcdsaKeyId {
      curve: EcdsaCurve::Secp256k1,
      name: key_name.clone(),
    }
  }).await;

  res.unwrap().0.signature
}
