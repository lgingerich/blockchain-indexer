// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use alloy_eips::eip7702::SignedAuthorization;
use alloy_primitives::{Address, Bloom, Bytes, FixedBytes, Uint};

#[derive(Debug, Clone)]
pub struct TransactionReceiptData {
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

#[derive(Debug, Clone)]
pub struct LogReceiptData {
    pub address: Address,
    pub topics: Vec<FixedBytes<32>>,
    pub data: Bytes,
    pub block_hash: Option<FixedBytes<32>>,
    pub block_number: Option<u64>,
    pub block_timestamp: Option<u64>,
    pub transaction_hash: Option<FixedBytes<32>>,
    pub transaction_index: Option<u64>,
    pub log_index: Option<u64>,
    pub removed: bool
}