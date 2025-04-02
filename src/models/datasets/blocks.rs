use alloy_primitives::{Address, Bloom, Bytes, FixedBytes};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

////////////////////////////////////// RPC Data ////////////////////////////////////////
// Base struct for common fields
#[derive(Debug, Clone)]
pub struct CommonRpcHeaderData {
    pub block_time: DateTime<Utc>,
    pub block_date: NaiveDate,
    pub block_number: u64,
    pub block_hash: FixedBytes<32>,
    pub parent_hash: FixedBytes<32>,
    pub nonce: Option<FixedBytes<8>>,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub base_fee_per_gas: Option<u64>,
    pub blob_gas_used: Option<u64>,
    pub excess_blob_gas: Option<u64>,
    pub extra_data: Bytes,
    pub difficulty: String,
    pub total_difficulty: Option<String>,
    pub size: Option<String>,
    pub miner: Address,
    pub logs_bloom: Bloom,
    pub sha3_uncles: FixedBytes<32>,
    pub receipts_root: FixedBytes<32>,
    pub state_root: FixedBytes<32>,
    pub transactions_root: FixedBytes<32>,
    pub withdrawals_root: Option<FixedBytes<32>>,
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
    pub l1_batch_number: Option<u64>,
    pub l1_batch_timestamp: Option<DateTime<Utc>>,
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
    pub block_time: DateTime<Utc>,
    pub block_date: NaiveDate,
    pub block_number: u64,
    pub block_hash: FixedBytes<32>,
    pub parent_hash: FixedBytes<32>,
    pub nonce: Option<FixedBytes<8>>,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub base_fee_per_gas: Option<u64>,
    pub blob_gas_used: Option<u64>,
    pub excess_blob_gas: Option<u64>,
    pub extra_data: Bytes,
    pub difficulty: String,
    pub total_difficulty: Option<String>,
    pub size: Option<String>,
    pub miner: Address,
    pub logs_bloom: Bloom,
    pub sha3_uncles: FixedBytes<32>,
    pub receipts_root: FixedBytes<32>,
    pub state_root: FixedBytes<32>,
    pub transactions_root: FixedBytes<32>,
    pub withdrawals_root: Option<FixedBytes<32>>,
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
    pub l1_batch_number: Option<u64>,
    pub l1_batch_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)] // Serialize without enum variant name
pub enum TransformedBlockData {
    Ethereum(EthereumTransformedBlockData),
    ZKsync(ZKsyncTransformedBlockData),
}
