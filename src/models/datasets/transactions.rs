#![allow(dead_code)] // Allow unused fields in RPC data for completeness

use alloy_eips::eip4844::BYTES_PER_BLOB;
use alloy_primitives::{Address, Bytes, FixedBytes};
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
    pub from_address: Address,
    pub to_address: TransactionTo,
    pub input: Option<Bytes>,
    pub value: Option<String>,
    pub gas_price: Option<u128>,
    pub gas_limit: u64,
    pub max_fee_per_gas: Option<u128>,
    pub max_priority_fee_per_gas: Option<u128>,
    pub effective_gas_price: Option<u128>,
    pub blob_versioned_hashes: Vec<FixedBytes<32>>,
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
    pub from_address: Address,
    pub to_address: Option<Address>,
    pub contract_address: Option<Address>,
    pub gas_used: u128,
    pub effective_gas_price: u128,
    pub cumulative_gas_used: u128,
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
}

#[derive(Debug, Clone)]
pub enum RpcTransactionReceiptData {
    Ethereum(EthereumRpcTransactionReceiptData),
    ZKsync(ZKsyncRpcTransactionReceiptData),
}

/////////////////////////////////// Transformed Data ///////////////////////////////////

#[derive(Debug, Serialize)]
pub struct CommonTransformedTransactionData {
    pub id: String,
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
    pub from_address: Address,
    pub to_address: Option<Address>,
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
    pub blob_versioned_hashes: Vec<FixedBytes<32>>,
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
}

#[derive(Debug, Serialize)]
#[serde(untagged)] // Serialize without enum variant name
pub enum TransformedTransactionData {
    Ethereum(EthereumTransformedTransactionData),
    ZKsync(ZKsyncTransformedTransactionData),
}
