use std::{
    fmt::{self, Display},
    str::FromStr,
};

use candid::{CandidType, Principal};
use ic_management_canister_types::{ DerivationPath, ECDSAPublicKeyArgs, ECDSAPublicKeyResponse, EcdsaCurve, EcdsaKeyId, SignWithECDSAReply, SignWithECDSAArgs};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

// https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#fees-for-the-t-schnorr-production-key
pub const ECDSA_SIGN_COST: u128 = 26_153_846_153;

#[derive(CandidType, Serialize, Debug)]
struct PublicKeyReply {
    pub public_key_hex: String,
}

#[derive(CandidType, Serialize, Debug)]
struct SignatureReply {
    pub signature_hex: String,
}

#[derive(CandidType, Serialize, Debug)]
struct SignatureVerificationReply {
    pub is_signature_valid: bool,
}


/// Fetches the ed25519 public key from the schnorr canister.
pub async fn ecdsa_public_key(key: EcdsaKey, derivation_path: Vec<ByteBuf>) -> Vec<u8> {
    let res: Result<(ECDSAPublicKeyResponse,), _> = ic_cdk::call(
        Principal::management_canister(),
        "schnorr_public_key",
        (ECDSAPublicKeyArgs {
            canister_id: None,
            derivation_path: DerivationPath::new(derivation_path),
            key_id: EcdsaKeyId {
                curve: EcdsaCurve::Secp256k1,
                name: key.to_string(),
            },
        },),
    )
    .await;

    res.expect("Failed to fetch ed25519 public key").0.public_key
}

/// Signs a message with an ed25519 key.
pub async fn sign_with_ecdsa(key: EcdsaKey, derivation_path: Vec<ByteBuf>, message_hash: [u8; 32]) -> Vec<u8> {
    ic_cdk::api::call::msg_cycles_accept128(ECDSA_SIGN_COST);

    let res: Result<(SignWithECDSAReply,), _> = ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        "sign_with_ecdsa",
        (SignWithECDSAArgs {
            message_hash,
            derivation_path: DerivationPath::new(derivation_path),
            key_id: EcdsaKeyId {
                curve: EcdsaCurve::Secp256k1,
                name: key.to_string(),
            },
        },),
        ECDSA_SIGN_COST as u64,
    )
    .await;

    res.expect("Failed to sign with ed25519").0.signature
}



#[derive(Debug, Clone, Deserialize, Serialize, CandidType)]
pub enum EcdsaKey {
    TestKeyLocalDevelopment,
    TestKey1,
    ProductionKey1,
    Custom(String),
}

impl Display for EcdsaKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let key_str = match self {
                Self::TestKeyLocalDevelopment => "dfx_test_key",
                Self::TestKey1 => "test_key_1",
                Self::ProductionKey1 => "key_1",
                Self::Custom(s) => s,
        };
        f.write_str(key_str)
    }
}

impl FromStr for EcdsaKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "dfx_test_key" => Self::TestKeyLocalDevelopment,
            "test_key_1" => Self::TestKey1,
            "key_1" => Self::ProductionKey1,
            _ => Self::Custom(s.to_string()),
        })
    }
}
