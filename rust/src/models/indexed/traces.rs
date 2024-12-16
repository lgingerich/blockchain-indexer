use alloy_primitives::{Address, Bytes, Uint};
use alloy_rpc_types_trace::geth::CallLogFrame;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TransformedTraceData {
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