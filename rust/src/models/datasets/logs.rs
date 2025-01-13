use alloy_primitives::{Address, Bytes, FixedBytes};
use serde::Serialize;

// Raw RPC response format
#[derive(Debug, Clone)]
pub struct RpcLogReceiptData {
    pub address: Address,
    pub topics: Vec<FixedBytes<32>>,
    pub data: Bytes,
    pub block_hash: Option<FixedBytes<32>>,
    pub block_number: Option<u64>,
    pub block_timestamp: Option<u64>,
    pub transaction_hash: Option<FixedBytes<32>>,
    pub transaction_index: Option<u64>,
    pub log_index: Option<u64>,
    pub removed: bool,
}

// Final output format
#[derive(Debug, Serialize)]
pub struct TransformedLogData {
    pub chain_id: u64,
    pub address: Address,
    pub topics: Vec<FixedBytes<32>>,
    pub data: Bytes,
    pub block_hash: Option<FixedBytes<32>>,
    pub block_number: Option<u64>,
    pub block_timestamp: Option<u64>,
    pub transaction_hash: Option<FixedBytes<32>>,
    pub transaction_index: Option<u64>,
    pub log_index: Option<u64>,
    pub removed: bool,
}
