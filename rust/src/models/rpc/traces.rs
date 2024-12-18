// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use alloy_primitives::{Address, Bytes, Uint};
use alloy_rpc_types_trace::geth::CallLogFrame;

#[derive(Debug, Clone)]
pub struct TraceData {
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
