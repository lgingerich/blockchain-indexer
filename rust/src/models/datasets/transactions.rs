use alloy_eips::eip2930::AccessList;
use alloy_eips::eip4844::BYTES_PER_BLOB;
use alloy_eips::eip7702::SignedAuthorization;
use alloy_primitives::{Address, Bloom, Bytes, FixedBytes, Uint};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

use crate::models::common::TransactionTo;

////////////////////////////////////// RPC Data ////////////////////////////////////////
///////////////////////////////// eth_getBlockByNumber /////////////////////////////////
#[derive(Debug, Clone)]
pub struct CommonRpcTransactionData {
    pub hash: FixedBytes<32>,
    pub nonce: u64,
    pub tx_type: u8,
    pub gas_price: u128,
    pub gas_limit: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    pub value: Option<Uint<256, 4>>,
    pub access_list: AccessList,
    pub input: Option<Bytes>,
    pub r: Uint<256, 4>,
    pub s: Uint<256, 4>,
    pub v: bool,
    pub blob_versioned_hashes: Vec<FixedBytes<32>>,
    pub authorization_list: Vec<SignedAuthorization>,
    pub block_hash: Option<FixedBytes<32>>,
    pub block_number: Option<u64>,
    pub transaction_index: Option<u64>,
    pub effective_gas_price: Option<u128>,
    pub from: Address,
    pub to: TransactionTo,
}

// Ethereum-specific transaction
#[derive(Debug, Clone)]
pub struct EthereumRpcTransactionData {
    pub common: CommonRpcTransactionData,
    pub max_fee_per_blob_gas: Option<u128>,
    pub blobs: Vec<FixedBytes<BYTES_PER_BLOB>>,
    pub commitments: Vec<FixedBytes<48>>,
    pub proofs: Vec<FixedBytes<48>>,
}

// ZKsync-specific transaction
#[derive(Debug, Clone)]
pub struct ZKsyncRpcTransactionData {
    pub common: CommonRpcTransactionData,
    // pub gas: u64, // TODO: Add back in
    pub l1_batch_number: Option<u64>,
    pub l1_batch_tx_index: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum RpcTransactionData {
    Ethereum(EthereumRpcTransactionData),
    ZKsync(ZKsyncRpcTransactionData),
}

///////////////////////////////// eth_getBlockReceipt //////////////////////////////////

#[derive(Debug, Clone)]
pub struct CommonRpcTransactionReceiptData {
    pub transaction_hash: FixedBytes<32>,
    pub transaction_index: Option<u64>,
    pub status: Option<bool>,
    pub tx_type: u8,
    pub block_hash: Option<FixedBytes<32>>,
    pub block_number: Option<u64>,
    pub gas_used: u128,
    pub effective_gas_price: u128,
    pub blob_gas_used: Option<u128>,
    pub blob_gas_price: Option<u128>,
    pub from: Address,
    pub to: Option<Address>,
    pub contract_address: Option<Address>,
    pub cumulative_gas_used: u128,
    pub authorization_list: Vec<SignedAuthorization>,
    pub logs_bloom: Bloom,
}

// Ethereum-specific receipt
#[derive(Debug, Clone)]
pub struct EthereumRpcTransactionReceiptData {
    pub common: CommonRpcTransactionReceiptData,
}

// ZKsync-specific receipt
#[derive(Debug, Clone)]
pub struct ZKsyncRpcTransactionReceiptData {
    pub common: CommonRpcTransactionReceiptData,
    pub l1_batch_number: Option<u64>,
    pub l1_batch_tx_index: Option<u64>,
    // pub l2_to_l1_logs: Option<Vec<L2ToL1Log>>, // TODO: Add back in
}

#[derive(Debug, Clone)]
pub enum RpcTransactionReceiptData {
    Ethereum(EthereumRpcTransactionReceiptData),
    ZKsync(ZKsyncRpcTransactionReceiptData),
}

/////////////////////////////////// Transformed Data ///////////////////////////////////

#[derive(Debug, Serialize)]
pub struct CommonTransformedTransactionData {
    pub chain_id: u64,

    // Block fields
    pub nonce: u64,
    pub gas_price: u128,
    pub gas_limit: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    pub value: Option<Uint<256, 4>>,
    pub access_list: AccessList,
    pub input: Option<Bytes>,
    pub r: Uint<256, 4>,
    pub s: Uint<256, 4>,
    pub v: bool,
    pub blob_versioned_hashes: Vec<FixedBytes<32>>,

    // Receipt fields
    pub transaction_hash: FixedBytes<32>,
    pub transaction_index: Option<u64>,
    pub status: Option<bool>,
    pub tx_type: u8,
    pub block_hash: Option<FixedBytes<32>>,
    pub block_number: Option<u64>,
    pub gas_used: u128,
    pub effective_gas_price: u128,
    pub blob_gas_used: Option<u128>,
    pub blob_gas_price: Option<u128>,
    pub from: Address,
    pub to: Option<Address>,
    pub contract_address: Option<Address>,
    pub cumulative_gas_used: u128,
    pub authorization_list: Vec<SignedAuthorization>,
    pub logs_bloom: Bloom,
}

#[derive(Debug, Serialize)]
pub struct EthereumTransformedTransactionData {
    #[serde(flatten)] // Flatten nested structs
    pub common: CommonTransformedTransactionData,
    pub max_fee_per_blob_gas: Option<u128>,
    pub blobs: Vec<FixedBytes<BYTES_PER_BLOB>>,
    pub commitments: Vec<FixedBytes<48>>,
    pub proofs: Vec<FixedBytes<48>>,
}

#[derive(Debug, Serialize)]
pub struct ZKsyncTransformedTransactionData {
    #[serde(flatten)] // Flatten nested structs
    pub common: CommonTransformedTransactionData,
    pub l1_batch_number: Option<u64>,
    pub l1_batch_tx_index: Option<u64>,
    // pub l2_to_l1_logs: Option<Vec<L2ToL1Log>>, // TODO: Add back in
}

#[derive(Debug, Serialize)]
#[serde(untagged)] // Serialize without enum variant name
pub enum TransformedTransactionData {
    Ethereum(EthereumTransformedTransactionData),
    ZKsync(ZKsyncTransformedTransactionData),
}
