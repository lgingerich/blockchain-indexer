use alloy_primitives::{Address, Bytes, Uint};
use alloy_rpc_types_trace::geth::CallLogFrame;
use serde::Serialize;

////////////////////////////////////// RPC Data ////////////////////////////////////////
// Raw RPC response format
#[derive(Debug, Clone)]
pub struct CommonRpcTraceData {
    pub from: Address,
    pub gas: Uint<256, 4>,
    pub gas_used: Uint<256, 4>,
    pub to: Option<Address>,
    pub input: Bytes,
    pub output: Option<Bytes>,
    pub error: Option<String>,
    pub revert_reason: Option<String>,
    pub logs: Vec<CallLogFrame>,
    pub value: Option<Uint<256, 4>>,
    pub typ: String,
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
    pub from: Address,
    pub gas: Uint<256, 4>,
    pub gas_used: Uint<256, 4>,
    pub to: Option<Address>,
    pub input: Bytes,
    pub output: Option<Bytes>,
    pub error: Option<String>,
    pub revert_reason: Option<String>,
    pub logs: Vec<CallLogFrame>,
    pub value: Option<Uint<256, 4>>,
    pub typ: String,
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
