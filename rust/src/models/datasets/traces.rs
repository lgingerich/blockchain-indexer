// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

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
    pub common: CommonTransformedTraceData,
}

// ZKsync-specific trace
#[derive(Debug, Clone, Serialize)]
pub struct ZKsyncTransformedTraceData {
    pub common: CommonTransformedTraceData,
}

#[derive(Debug, Clone)]
pub enum TransformedTraceData {
    Ethereum(EthereumTransformedTraceData),
    ZKsync(ZKsyncTransformedTraceData),
}