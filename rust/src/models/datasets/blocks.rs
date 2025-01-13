use alloy_primitives::{Address, Bloom, Bytes, FixedBytes, Uint};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

// Raw RPC response format
#[derive(Debug, Clone)]
pub struct RpcHeaderData {
    pub hash: FixedBytes<32>,
    pub parent_hash: FixedBytes<32>,
    pub ommers_hash: FixedBytes<32>,
    pub beneficiary: Address,
    pub state_root: FixedBytes<32>,
    pub transactions_root: FixedBytes<32>,
    pub receipts_root: FixedBytes<32>,
    pub logs_bloom: Bloom,
    pub difficulty: Uint<256, 4>,
    pub block_number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub block_time: DateTime<Utc>,
    pub block_date: NaiveDate,
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

// Final output format
#[derive(Debug, Serialize)]
pub struct TransformedBlockData {
    pub chain_id: u64,
    pub hash: FixedBytes<32>,
    pub parent_hash: FixedBytes<32>,
    pub ommers_hash: FixedBytes<32>,
    pub beneficiary: Address,
    pub state_root: FixedBytes<32>,
    pub transactions_root: FixedBytes<32>,
    pub receipts_root: FixedBytes<32>,
    pub logs_bloom: Bloom,
    pub difficulty: Uint<256, 4>,
    pub block_number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub block_time: DateTime<Utc>,
    pub block_date: NaiveDate,
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


// Create type alias for alloy Withdrawal type
// Do not expect to need custom modifications to this type
pub type RpcWithdrawalData = alloy_rpc_types_eth::Withdrawal;