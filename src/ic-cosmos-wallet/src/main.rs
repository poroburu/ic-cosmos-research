use std::str::FromStr;

use candid::candid_method;
use ic_cdk::update;

use ic_cosmos::{
    rpc_client::{RpcConfig, RpcResult, RpcServices},
    types::{BlockHash, Pubkey, RpcSendTransactionConfig, Transaction},
};

use ic_cosmos_wallet::{
    ecdsa::{ecdsa_public_key, sign_with_ecdsa},
    state::{read_state, InitArgs, State},
    utils::validate_caller_not_anonymous,
};

use serde_bytes::ByteBuf;

/// Returns the public key of the Cosmos wallet associated with the caller.
///
/// # Returns
///
/// - `String`: The Cosmos public key as a string.
#[update]
#[candid_method]
pub async fn address() -> String {
    let caller = validate_caller_not_anonymous();
    let key_name = read_state(|s| s.ecdsa_key.to_owned());
    let derived_path = vec![ByteBuf::from(caller.as_slice())];
    let pk = ecdsa_public_key(key_name, derived_path).await;
    Pubkey::try_from(pk.as_slice())
        .expect("Invalid public key")
        .to_string()
}

/// Signs a provided message using the caller's Eddsa key.
///
/// # Parameters
///
/// - `message` (`String`): The message to be signed.
///
/// # Returns
///
/// - `RpcResult<String>`: The signature as a base58 encoded string on success, or an `RpcError` on
///   failure.
#[update(name = "signMessage")]
#[candid_method(query, rename = "signMessage")]
pub async fn sign_message(message: String) -> Vec<u8> {
    let caller = validate_caller_not_anonymous();
    let key_name = read_state(|s| s.ecdsa_key.to_owned());
    let derived_path = vec![ByteBuf::from(caller.as_slice())];
    sign_with_ecdsa(
        key_name,
        derived_path,
        message
            .as_bytes()
            .try_into()
            .expect("Failed to convert message to 32 bytes"),
    )
    .await
}

/// Signs and sends a transaction to the Cosmos network.
///
/// # Parameters
///
/// - `provider` (`String`): The Cosmos RPC provider ID.
/// - `raw_transaction` (`String`): The serialized unsigned transaction.
/// - `config` (`Option<RpcSendTransactionConfig>`): Optional configuration for sending the
///   transaction.
///
/// # Returns
///
/// - `RpcResult<String>`: The transaction signature as a string on success, or an `RpcError` on
///   failure.
#[update(name = "sendTransaction")]
#[candid_method(query, rename = "sendTransaction")]
pub async fn send_transaction(
    source: RpcServices,
    config: Option<RpcConfig>,
    raw_transaction: String,
    params: Option<RpcSendTransactionConfig>,
) -> RpcResult<String> {
    let caller = validate_caller_not_anonymous();
    let cos_canister = read_state(|s| s.cos_canister);

    let mut tx = Transaction::from_str(&raw_transaction).expect("Invalid transaction");

    if tx.message.recent_blockhash == BlockHash::default() {
        let response = ic_cdk::call::<_, (RpcResult<String>,)>(
            cos_canister,
            "cos_getLatestBlockhash",
            (&source,),
        )
        .await?;
        tx.message.recent_blockhash =
            BlockHash::from_str(&response.0?).expect("Invalid recent blockhash")
    }

    let key_name = read_state(|s| s.ecdsa_key.to_owned());
    let derived_path = vec![ByteBuf::from(caller.as_slice())];

    let signature = sign_with_ecdsa(
        key_name,
        derived_path,
        tx.message_data()
            .try_into()
            .expect("Failed to convert message to 32 bytes"),
    )
    .await
    .try_into()
    .expect("Invalid signature");

    tx.add_signature(0, signature);

    let response = ic_cdk::call::<_, (RpcResult<String>,)>(
        cos_canister,
        "cos_sendTransaction",
        (&source, config, tx.to_string(), params),
    )
    .await?;

    response.0
}

#[ic_cdk::init]
fn init(args: InitArgs) {
    State::init(args);
}

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    State::pre_upgrade();
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: Option<InitArgs>) {
    State::post_upgrade(args);
}

fn main() {}
ic_cdk::export_candid!();
