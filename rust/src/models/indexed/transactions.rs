use alloy_eips::eip2930::AccessList;
use alloy_eips::eip4844::BYTES_PER_BLOB;
use alloy_eips::eip7702::SignedAuthorization;
use alloy_primitives::{Address, Bloom, Bytes, FixedBytes, TxKind, Uint};

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

use crate::models::common::{ChainId, TransactionTo};

// TODO: Verify fields and cleanup
#[derive(Debug, Serialize)]
pub struct TransformedTransactionData {

    // Block fields
    pub chain_id: u64,
    // pub hash: FixedBytes<32>,
    pub nonce: u64,
    pub gas_price: u128,
    pub gas_limit: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    // pub from: Address,
    // pub to: TransactionTo,
    pub value: Uint<256, 4>,
    pub access_list: Option<AccessList>,
    // pub authorization_list: Vec<SignedAuthorization>,
    pub blob_versioned_hashes: Vec<FixedBytes<32>>,
    pub max_fee_per_blob_gas: u128,
    pub blobs: Vec<FixedBytes<BYTES_PER_BLOB>>,
    pub commitments: Vec<FixedBytes<48>>,
    pub proofs: Vec<FixedBytes<48>>,
    pub input: Bytes,
    pub r: Uint<256, 4>,
    pub s: Uint<256, 4>,
    pub v: bool,
    // pub block_hash: Option<FixedBytes<32>>,
    // pub block_number: Option<u64>,
    // pub transaction_index: Option<u64>,
    // pub effective_gas_price: Option<u128>,

    // Receipt fields
    pub status: Option<bool>,
    pub cumulative_gas_used: u128,
    pub logs_bloom: Bloom,
    pub transaction_hash: FixedBytes<32>,
    pub transaction_index: Option<u64>,
    pub block_hash: Option<FixedBytes<32>>,
    pub block_number: Option<u64>,
    pub gas_used: u128,
    pub effective_gas_price: u128,
    pub blob_gas_used: Option<u128>,
    pub blob_gas_price: Option<u128>,
    pub from: Address,
    pub to: Option<Address>,
    pub contract_address: Option<Address>,
    pub authorization_list: Option<Vec<SignedAuthorization>>
}