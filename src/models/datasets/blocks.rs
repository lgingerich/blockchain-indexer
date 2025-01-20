use alloy_primitives::{Address, Bloom, Bytes, FixedBytes, Uint};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

////////////////////////////////////// RPC Data ////////////////////////////////////////
// Base struct for common fields
#[derive(Debug, Clone)]
pub struct CommonRpcHeaderData {
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
    pub mix_hash: Option<FixedBytes<32>>,
    pub nonce: Option<FixedBytes<8>>,
    pub base_fee_per_gas: Option<u64>,
    pub withdrawals_root: Option<FixedBytes<32>>,
    pub blob_gas_used: Option<u64>,
    pub excess_blob_gas: Option<u64>,
    pub parent_beacon_block_root: Option<FixedBytes<32>>,
    pub requests_hash: Option<FixedBytes<32>>,
    pub total_difficulty: Option<Uint<256, 4>>,
    pub size: Option<Uint<256, 4>>,
}

// Ethereum-specific header
#[derive(Debug, Clone)]
pub struct EthereumRpcHeaderData {
    pub common: CommonRpcHeaderData,
}

// ZKsync-specific header
#[derive(Debug, Clone)]
pub struct ZKsyncRpcHeaderData {
    pub common: CommonRpcHeaderData,
    pub target_blobs_per_block: Option<u64>,
    pub l1_batch_number: Option<u64>,
    pub l1_batch_timestamp: Option<DateTime<Utc>>,
    // pub seal_fields: Option<Vec<String>>, // TODO: Add this back in
}

#[derive(Debug, Clone)]
pub enum RpcHeaderData {
    Ethereum(EthereumRpcHeaderData),
    ZKsync(ZKsyncRpcHeaderData),
}

/////////////////////////////////// Transformed Data ///////////////////////////////////

// Base struct for common fields
#[derive(Debug, Clone, Serialize)]
pub struct CommonTransformedBlockData {
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
    pub mix_hash: Option<FixedBytes<32>>,
    pub nonce: Option<FixedBytes<8>>,
    pub base_fee_per_gas: Option<u64>,
    pub withdrawals_root: Option<FixedBytes<32>>,
    pub blob_gas_used: Option<u64>,
    pub excess_blob_gas: Option<u64>,
    pub parent_beacon_block_root: Option<FixedBytes<32>>,
    pub requests_hash: Option<FixedBytes<32>>,
    pub total_difficulty: Option<Uint<256, 4>>,
    pub size: Option<Uint<256, 4>>,
}

// Ethereum-specific header
#[derive(Debug, Clone, Serialize)]
pub struct EthereumTransformedBlockData {
    #[serde(flatten)] // Flatten nested structs
    pub common: CommonTransformedBlockData,
}

// ZKsync-specific header
#[derive(Debug, Clone, Serialize)]
pub struct ZKsyncTransformedBlockData {
    #[serde(flatten)] // Flatten nested structs
    pub common: CommonTransformedBlockData,
    pub target_blobs_per_block: Option<u64>,
    pub l1_batch_number: Option<u64>,
    pub l1_batch_timestamp: Option<DateTime<Utc>>,
    // pub seal_fields: Option<Vec<String>>, // TODO: Add this back in
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)] // Serialize without enum variant name
pub enum TransformedBlockData {
    Ethereum(EthereumTransformedBlockData),
    ZKsync(ZKsyncTransformedBlockData),
}
