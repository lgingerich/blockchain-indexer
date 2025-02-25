use alloy_primitives::FixedBytes;
use alloy_rpc_types_trace::geth::{CallFrame, GethTrace, TraceResult};
use anyhow::Result;
use serde_json::Value;

use crate::models::common::Chain;
use crate::models::datasets::traces::{
    CommonRpcTraceData, EthereumRpcTraceData, RpcTraceData, ZKsyncRpcTraceData,
};

pub trait TraceParser {
    fn parse_traces(self, chain: Chain, block_number: u64) -> Result<Vec<RpcTraceData>>;
}
// TODO: Implement all tracer methods
impl TraceParser for Vec<TraceResult> {
    fn parse_traces(self, chain: Chain, block_number: u64) -> Result<Vec<RpcTraceData>> {
        let mut results = Vec::new();
        
        for trace_result in self {
            match trace_result {
                TraceResult::Success { result, tx_hash } => {
                    match result {
                        GethTrace::Default(_frame) => {
                            unimplemented!()
                        }
                        GethTrace::CallTracer(frame) => {
                            let mut stack = vec![(frame, tx_hash, Vec::new())];
                            
                            while let Some((current_frame, current_tx_hash, trace_address)) = stack.pop() {
                                // Process current frame
                                let trace_data = create_trace_data(
                                    &current_frame,
                                    current_tx_hash, 
                                    chain, 
                                    block_number, 
                                    &trace_address
                                );
                                results.push(trace_data);
                                
                                // Add child frames to stack in reverse order (to maintain DFS order)
                                // This avoids unnecessary cloning of trace_address for each child
                                let mut child_address = trace_address.clone();
                                for (i, child) in current_frame.calls.into_iter().enumerate().rev() {
                                    child_address.push(i);
                                    stack.push((child, current_tx_hash, child_address.clone()));
                                    child_address.pop(); // Remove the last element for the next iteration
                                }
                            }
                        }
                        GethTrace::FlatCallTracer(_frame) => {
                            unimplemented!()
                        }
                        GethTrace::FourByteTracer(_frame) => {
                            unimplemented!()
                        }
                        GethTrace::PreStateTracer(_frame) => {
                            unimplemented!()
                        }
                        GethTrace::NoopTracer(_frame) => {
                            unimplemented!()
                        }
                        GethTrace::MuxTracer(_frame) => {
                            unimplemented!()
                        }
                        GethTrace::JS(frame) => {
                            match frame {
                                Value::Null => {
                                    // Return empty vector to continue processing
                                    return Ok(Vec::new());
                                }
                                Value::Bool(_bool) => {
                                    unimplemented!()
                                }
                                Value::Number(_number) => {
                                    unimplemented!()
                                }
                                Value::String(_string) => {
                                    unimplemented!()
                                }
                                Value::Array(_arr) => {
                                    unimplemented!()
                                }
                                Value::Object(_obj) => {
                                    unimplemented!()
                                }
                            }
                        }
                    }
                }
                TraceResult::Error { error, tx_hash } => {
                    // Log failed traces with their error messages
                    if let Some(hash) = tx_hash {
                        tracing::warn!(
                            "Failed to process trace for transaction {}: {}",
                            hash,
                            error
                        );
                    } else {
                        tracing::warn!("Failed to process trace: {}", error);
                    }
                }
            }
        }
        
        Ok(results)
    }
}

// Helper function to create trace data without cloning the entire frame
fn create_trace_data(
    frame: &CallFrame,
    tx_hash: Option<FixedBytes<32>>,
    chain: Chain,
    block_number: u64,
    trace_address: &[usize]
) -> RpcTraceData {
    let common_data = CommonRpcTraceData {
        block_number,
        tx_hash,
        r#type: frame.typ.to_lowercase(),
        trace_address: trace_address.to_vec(),
        from: frame.from,
        to: frame.to,
        value: frame.value.map(|v| v.to_string()),
        gas: frame.gas.to_string(),
        gas_used: frame.gas_used.to_string(),
        input: frame.input.clone(),
        output: frame.output.clone(),
        error: frame.error.clone(),
        revert_reason: frame.revert_reason.clone(),
        logs: frame.logs.clone(),
    };

    match chain {
        Chain::Ethereum => RpcTraceData::Ethereum(EthereumRpcTraceData {
            common: common_data,
        }),
        Chain::ZKsync => RpcTraceData::ZKsync(ZKsyncRpcTraceData {
            common: common_data,
        }),
    }
}
