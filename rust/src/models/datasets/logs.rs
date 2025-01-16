use alloy_primitives::{Address, Bytes, FixedBytes};
use serde::Serialize;


////////////////////////////////////// RPC Data ////////////////////////////////////////
// Base struct for common fields
#[derive(Debug, Clone)]
pub struct CommonRpcLogReceiptData {
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

// Ethereum-specific log receipt
#[derive(Debug, Clone)]
pub struct EthereumRpcLogReceiptData {
    pub common: CommonRpcLogReceiptData,
}

// ZKsync-specific log receipt
#[derive(Debug, Clone)]
pub struct ZKsyncRpcLogReceiptData {
    pub common: CommonRpcLogReceiptData,
}

#[derive(Debug, Clone)]
pub enum RpcLogReceiptData {
    Ethereum(EthereumRpcLogReceiptData),
    ZKsync(ZKsyncRpcLogReceiptData),
}

/////////////////////////////////// Transformed Data ///////////////////////////////////
// Final output format
#[derive(Debug, Clone, Serialize)]
pub struct CommonTransformedLogData {
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

// Ethereum-specific log receipt
#[derive(Debug, Clone)]
pub struct EthereumTransformedLogData {
    pub common: CommonTransformedLogData,
}

// ZKsync-specific log receipt
#[derive(Debug, Clone)]
pub struct ZKsyncTransformedLogData {
    pub common: CommonTransformedLogData,
}

#[derive(Debug, Clone)]
pub enum TransformedLogData {
    Ethereum(EthereumTransformedLogData),
    ZKsync(ZKsyncTransformedLogData),
}

