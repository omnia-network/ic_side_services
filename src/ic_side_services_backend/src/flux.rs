use std::cell::RefCell;

use candid::CandidType;
use ic_cdk::trap;
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::{ecdsa_api, logger::log};

// Bitcoin message signature:
// - Rust: https://docs.rs/bitcoin/latest/src/bitcoin/sign_message.rs.html#197-204
// - JS: https://github.com/bitcoinjs/bitcoinjs-message

thread_local! {
    /// The ECDSA canister public key, obtained from [ecdsa_api::ecdsa_public_key].
    /* flexible */ static CANISTER_ECDSA_PUBLIC_KEY: RefCell<Option<Vec<u8>>> = RefCell::new(None);
}

#[derive(CandidType, Serialize, Deserialize, Debug, Copy, Clone)]
pub enum FluxNetwork {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "testnet")]
    Testnet,
    #[serde(rename = "mainnet")]
    Mainnet,
}

pub enum P2PKHAddress {
    /// Used to login in Flux.
    ///
    /// There's no good and clear documentation from Flux,
    /// but this could help: https://github.com/RunOnFlux/flux/blob/a06e3d4a282da8e871339c05f5195725679763f7/ZelBack/src/services/idService.js#L227.
    ZelId,
    /// Used for sending and receiving funds on Flux,
    /// which is based on zcash.
    ///
    /// For public key to address, see:
    /// - https://zips.z.cash/protocol/protocol.pdf#addressandkeyencoding
    /// - https://forum.zcashcommunity.com/t/zcash-wif-prefixes/38714/4
    ZCash,
}

pub async fn set_canister_ecdsa_public_key(key_name: String, derivation_path: Vec<Vec<u8>>) {
    // Fetch the public key of the given derivation path.
    let public_key = ecdsa_api::ecdsa_public_key(key_name.clone(), derivation_path.clone()).await;
    CANISTER_ECDSA_PUBLIC_KEY.with(|k| {
        k.replace(Some(public_key.clone()));
    });

    log(&format!(
        "set_canister_ecdsa_public_key: key_name: {}, derivation_path: {:?}, public_key: {:?}",
        key_name, derivation_path, public_key
    ));
}

/// Returns the P2PKH address of this canister at the given derivation path.
pub fn get_p2pkh_address(network: FluxNetwork, address_type: P2PKHAddress) -> String {
    // Fetch the public key of the given derivation path.
    let public_key = CANISTER_ECDSA_PUBLIC_KEY.with(|k| k.borrow().clone());

    if public_key.is_none() {
        trap(
            "Canister ECDSA public key is not set. Call the set_canister_public_key method first.",
        );
    }

    // Compute the address.
    public_key_to_p2pkh_address(network, address_type, &public_key.unwrap())
}

fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}
fn hash256(data: &[u8]) -> Vec<u8> {
    sha256(&sha256(data))
}
fn ripemd160(data: &[u8]) -> Vec<u8> {
    let mut hasher = ripemd::Ripemd160::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Converts a public key to a P2PKH address.
fn public_key_to_p2pkh_address(
    network: FluxNetwork,
    address_type: P2PKHAddress,
    public_key: &[u8],
) -> String {
    // SHA-256 & RIPEMD-160
    let result = ripemd160(&sha256(public_key));

    let mut data_with_prefix = match network {
        FluxNetwork::Local | FluxNetwork::Testnet => match address_type {
            P2PKHAddress::ZelId => vec![0x6f],
            P2PKHAddress::ZCash => vec![0x1d, 0x25],
        },
        FluxNetwork::Mainnet => match address_type {
            P2PKHAddress::ZelId => vec![0x00],
            P2PKHAddress::ZCash => vec![0x1c, 0xb8],
        },
    };

    data_with_prefix.extend(result);

    let checksum = &sha256(&sha256(&data_with_prefix.clone()))[..4];

    let mut full_address = data_with_prefix;
    full_address.extend(checksum);

    bs58::encode(full_address).into_string()
}

// see https://github.com/bitcoinjs/varuint-bitcoin/blob/8342fe7362f20a412d61b9ade20839aafaa7f78e/index.js#L79-L88
fn encoding_length(l: usize) -> usize {
    if l < 0xfd {
        1
    } else if l <= 0xffff {
        3
    } else if l <= 0xffffffff {
        5
    } else {
        9
    }
}

// see https://github.com/bitcoinjs/varuint-bitcoin/blob/master/index.js#L11-L44
fn encode_varuint(number: usize) -> Vec<u8> {
    if number < 0xfd {
        vec![(number as u8)]
    } else if number <= 0xffff {
        let mut buf = vec![0xfd];
        buf.extend_from_slice(&(number as u16).to_le_bytes());
        buf
    } else if number <= 0xffffffff {
        let mut buf = vec![0xfe];
        buf.extend_from_slice(&(number as u32).to_le_bytes());
        buf
    } else {
        let mut buf = vec![0xff];
        buf.extend_from_slice(&(number >> 0 as u32).to_le_bytes());
        buf.extend_from_slice(&((((number as u64) / 0x100000000) | 0) as u32).to_le_bytes());
        buf
    }
}

// see https://github.com/bitcoinjs/bitcoinjs-message/blob/c43430f4c03c292c719e7801e425d887cbdf7464/index.js#L57-L73
pub fn get_message_magic_hash(message: String) -> Vec<u8> {
    let msg_prefix = b"\x18Bitcoin Signed Message:\n";
    let message_vi_size = encoding_length(message.len());
    let mut buffer = Vec::<u8>::with_capacity(msg_prefix.len() + message_vi_size + message.len());
    // insert prefix
    buffer.extend_from_slice(msg_prefix);
    // insert message length
    buffer.extend(encode_varuint(message.len()));
    // insert message
    buffer.extend_from_slice(message.as_bytes());

    hash256(&buffer)
}

// see https://github.com/bitcoinjs/bitcoinjs-message/blob/c43430f4c03c292c719e7801e425d887cbdf7464/index.js#L27-L35
pub fn encode_signature(signature_bytes: &[u8]) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![5 + 27]; // flagByte
    bytes.extend_from_slice(signature_bytes);
    bytes
}
