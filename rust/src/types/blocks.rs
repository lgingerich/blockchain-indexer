// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use chrono::{DateTime, NaiveDate,Utc};
use alloy_primitives::{Address, Bloom, Bytes, FixedBytes, Uint};
use alloy_eips::eip2930::AccessList;

#[derive(Debug)]
pub struct HeaderData {
    pub hash: FixedBytes<32>,
    pub parent_hash: FixedBytes<32>,
    pub ommers_hash: FixedBytes<32>,
    pub beneficiary: Address,
    pub state_root: FixedBytes<32>,
    pub transactions_root: FixedBytes<32>,
    pub receipts_root: FixedBytes<32>,
    pub logs_bloom: Bloom,
    pub difficulty: Uint<256, 4>,
    pub number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: DateTime<Utc>,
    pub date: NaiveDate,
    pub extra_data: Bytes,
    pub mix_hash: FixedBytes<32>,
    pub nonce: FixedBytes<8>,
    pub base_fee_per_gas: Option<u64>,
    pub withdrawals_root: Option<FixedBytes<32>>,
    pub blob_gas_used: Option<u64>,
    pub excess_blob_gas: Option<u64>,
    pub parent_beacon_block_root: Option<FixedBytes<32>>,
    pub requests_hash: Option<FixedBytes<32>>,
    pub target_blobs_per_block: Option<u64>,
    pub total_difficulty: Option<Uint<256, 4>>,
    pub size: Option<Uint<256, 4>>,
}

#[derive(Debug)]
pub struct TransactionData {
    pub nonce: u64,
    pub gas_limit: u64,
    pub value: Uint<256, 4>,
    pub input: Bytes,

}
