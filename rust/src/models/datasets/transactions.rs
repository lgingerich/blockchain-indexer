use alloy_eips::eip2930::AccessList;
use alloy_eips::eip4844::BYTES_PER_BLOB;
use alloy_eips::eip7702::SignedAuthorization;
use alloy_primitives::{Address, Bloom, Bytes, FixedBytes, Uint};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

use crate::models::common::TransactionTo;

// Raw RPC response format from `eth_getBlockByNumber`
#[derive(Debug, Clone)]
pub struct RpcTransactionData {
    pub nonce: u64,
    pub gas_price: u128,
    pub gas_limit: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    pub to: TransactionTo,
    pub value: Uint<256, 4>,
    pub access_list: AccessList,
    pub authorization_list: Vec<SignedAuthorization>,
    pub blob_versioned_hashes: Vec<FixedBytes<32>>,
    pub max_fee_per_blob_gas: u128,
    pub blobs: Vec<FixedBytes<BYTES_PER_BLOB>>,
    pub commitments: Vec<FixedBytes<48>>,
    pub proofs: Vec<FixedBytes<48>>,
    pub input: Bytes,
    pub r: Uint<256, 4>,
    pub s: Uint<256, 4>,
    pub v: bool,
    pub hash: FixedBytes<32>,

    pub block_hash: Option<FixedBytes<32>>,
    pub block_number: Option<u64>,
    pub transaction_index: Option<u64>,
    pub effective_gas_price: Option<u128>,
    pub from: Address,


    pub tx_type: u8,
    // pub gas: 

    // ZKsync fields
    // Sometimes the below fields are in inner and other times in other
    pub l1_batch_number: Option<u64>,
    pub l1_batch_tx_index: Option<u64>,
    // pub max_fee_per_gas: Option<u128>,
    // pub max_priority_fee_per_gas: Option<u128>,
}

// Raw RPC response format from `eth_getTransactionReceipt`
#[derive(Debug, Clone)]
pub struct RpcTransactionReceiptData {
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
    pub authorization_list: Option<Vec<SignedAuthorization>>,
    pub tx_type: u8,

    // ZKsync fields
    pub l1_batch_number: Option<u64>,
    pub l1_batch_tx_index: Option<u64>,
    // pub l2_to_l1_logs: Option<Vec<L2ToL1Log>>,
}

// Final output format
// TODO: Verify fields and cleanup
#[derive(Debug, Serialize)]
pub struct TransformedTransactionData {
    // Block fields
    pub chain_id: u64,
    pub tx_type: u8,
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
    // pub authorization_list: Option<Vec<SignedAuthorization>> // TODO: Implement this. Need to handle private fields and updating the BigQuery schema

    // ZKsync fields
    pub l1_batch_number: Option<u64>,
    pub l1_batch_tx_index: Option<u64>,
}
