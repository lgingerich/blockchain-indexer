#![allow(dead_code)] // Allow unused fields in RPC data for completeness

use alloy_eips::{eip2930::AccessList, eip4844::BYTES_PER_BLOB, eip7702::SignedAuthorization};
use alloy_primitives::{Address, Bloom, Bytes, FixedBytes};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

use crate::models::common::TransactionTo;

////////////////////////////////////// RPC Data ////////////////////////////////////////
///////////////////////////////// eth_getBlockByNumber /////////////////////////////////
#[derive(Debug, Clone)]
pub struct CommonRpcTransactionData {
    pub block_number: Option<u64>,
    pub block_hash: Option<FixedBytes<32>>,
    pub tx_hash: FixedBytes<32>,
    pub tx_index: Option<u64>,
    pub tx_type: u8,
    pub nonce: u64,
    pub from: Address,
    pub to: TransactionTo,
    pub input: Option<Bytes>,
    pub value: Option<String>,
    pub gas_price: Option<u128>,
    pub gas_limit: u64,
    pub max_fee_per_gas: Option<u128>,
    pub max_priority_fee_per_gas: Option<u128>,
    pub effective_gas_price: Option<u128>,
    pub access_list: AccessList,
    pub authorization_list: Vec<SignedAuthorization>,
    pub blob_versioned_hashes: Vec<FixedBytes<32>>,
    pub r: Option<String>,
    pub s: Option<String>,
    pub v: Option<bool>,
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
    pub block_number: Option<u64>,
    pub block_hash: Option<FixedBytes<32>>,
    pub tx_hash: FixedBytes<32>,
    pub tx_index: Option<u64>,
    pub tx_type: u8,
    pub status: Option<bool>,
    pub from: Address,
    pub to: Option<Address>,
    pub contract_address: Option<Address>,
    pub gas_used: u128,
    pub effective_gas_price: u128,
    pub cumulative_gas_used: u128,
    pub blob_gas_price: Option<u128>,
    pub blob_gas_used: Option<u128>,
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
    pub block_time: DateTime<Utc>,
    pub block_date: NaiveDate,
    pub block_number: Option<u64>,
    pub block_hash: Option<FixedBytes<32>>,
    pub tx_hash: FixedBytes<32>,
    pub tx_index: Option<u64>,
    pub tx_type: u8,
    pub status: Option<bool>,
    pub nonce: u64,
    pub from: Address,
    pub to: Option<Address>,
    pub contract_address: Option<Address>,
    pub input: Option<Bytes>,
    pub value: Option<String>,
    pub gas_price: Option<u128>,
    pub gas_limit: u64,
    pub gas_used: u128,
    pub max_fee_per_gas: Option<u128>,
    pub max_priority_fee_per_gas: Option<u128>,
    pub effective_gas_price: u128,
    pub cumulative_gas_used: u128,
    pub blob_gas_price: Option<u128>,
    pub blob_gas_used: Option<u128>,
    pub access_list: AccessList,
    pub authorization_list: Vec<SignedAuthorization>,
    pub blob_versioned_hashes: Vec<FixedBytes<32>>,
    pub logs_bloom: Bloom,
    pub r: Option<String>,
    pub s: Option<String>,
    pub v: Option<bool>,    
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
