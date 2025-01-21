use alloy_primitives::{Address, Bytes, FixedBytes};
use alloy_rpc_types_trace::geth::CallLogFrame;
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

////////////////////////////////////// RPC Data ////////////////////////////////////////
// Raw RPC response format
#[derive(Debug, Clone)]
pub struct CommonRpcTraceData {
    pub block_number: u64,
    pub tx_hash: Option<FixedBytes<32>>,
    pub r#type: String,
    pub from: Address,
    pub to: Option<Address>,
    pub value: Option<String>,
    pub gas: String,
    pub gas_used: String,
    pub input: Bytes,
    pub output: Option<Bytes>,
    pub error: Option<String>,
    pub revert_reason: Option<String>,
    pub logs: Vec<CallLogFrame>,
}

// Ethereum-specific trace
#[derive(Debug, Clone)]
pub struct EthereumRpcTraceData {
    pub common: CommonRpcTraceData,
}

// ZKsync-specific trace
#[derive(Debug, Clone)]
pub struct ZKsyncRpcTraceData {
    pub common: CommonRpcTraceData,
}

#[derive(Debug, Clone)]
pub enum RpcTraceData {
    Ethereum(EthereumRpcTraceData),
    ZKsync(ZKsyncRpcTraceData),
}

/////////////////////////////////// Transformed Data ///////////////////////////////////
#[derive(Debug, Clone, Serialize)]
pub struct CommonTransformedTraceData {
    pub chain_id: u64,
    pub block_time: DateTime<Utc>,
    pub block_date: NaiveDate,
    pub block_number: u64,
    pub tx_hash: Option<FixedBytes<32>>,
    pub r#type: String,
    pub from: Address,
    pub to: Option<Address>,
    pub value: Option<String>,
    pub gas: String,
    pub gas_used: String,
    pub input: Bytes,
    pub output: Option<Bytes>,
    pub error: Option<String>,
    pub revert_reason: Option<String>,
    pub logs: Vec<CallLogFrame>,
}

// Ethereum-specific trace
#[derive(Debug, Clone, Serialize)]
pub struct EthereumTransformedTraceData {
    #[serde(flatten)] // Flatten nested structs
    pub common: CommonTransformedTraceData,
}

// ZKsync-specific trace
#[derive(Debug, Clone, Serialize)]
pub struct ZKsyncTransformedTraceData {
    #[serde(flatten)] // Flatten nested structs
    pub common: CommonTransformedTraceData,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)] // Serialize without enum variant name
pub enum TransformedTraceData {
    Ethereum(EthereumTransformedTraceData),
    ZKsync(ZKsyncTransformedTraceData),
}
