use alloy_primitives::FixedBytes;
use alloy_rpc_types_trace::geth::{CallFrame, GethTrace, TraceResult};
use anyhow::Result;
use serde_json::Value;

use crate::models::{
    common::Chain,
    datasets::traces::{
        CommonRpcTraceData, EthereumRpcTraceData, RpcTraceData, ZKsyncRpcTraceData,
    },
};

pub trait TraceParser {
    fn parse_traces(self, chain_info: &ChainInfo, block_number: u64) -> Result<Vec<RpcTraceData>>;
}

impl TraceParser for Vec<TraceResult> {
    fn parse_traces(self, chain_info: &ChainInfo, block_number: u64) -> Result<Vec<RpcTraceData>> {
        Ok(self
            .into_iter()
            .flat_map(|trace_result| {
                match trace_result {
                    TraceResult::Success { result, tx_hash } => {
                        // TODO: Add support for other trace types
                        match result {
                            GethTrace::Default(_frame) => {
                                unimplemented!()
                            }
                            GethTrace::CallTracer(frame) => {
                                // Process the frame and all its nested calls
                                flatten_call_frames(frame, tx_hash, chain_info, block_number)
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
                                        Vec::new()
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

                    // TODO: Should I be using `error` for the `error` or `revert_reason` fields?
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
                        Vec::new()
                    }
                }
            })
            .collect())
    }
}

// Recursively flattens a CallFrame and its nested calls into a vector of TraceData
fn flatten_call_frames(
    frame: CallFrame,
    tx_hash: Option<FixedBytes<32>>,
    chain_info: &ChainInfo,
    block_number: u64,
) -> Vec<RpcTraceData> {
    // Helper function to process frame and its children with trace address
    fn process_frame_with_address(
        frame: CallFrame,
        tx_hash: Option<FixedBytes<32>>,
        chain_info: &ChainInfo,
        block_number: u64,
        trace_address: Vec<usize>,
    ) -> Vec<RpcTraceData> {
        let mut traces = Vec::new();

        let common_data = CommonRpcTraceData {
            block_number,
            tx_hash,
            trace_type: frame.typ.to_lowercase(),
            subtraces: frame.calls.len(),
            trace_address: trace_address.to_owned(),
            from_address: frame.from,
            to_address: frame.to,
            value: frame.value.map(|v| v.to_string()), // Convert from Uint<256, 4> to String for proper serialization
            gas: frame.gas.to_string(), // Convert from Uint<256, 4> to String for proper serialization
            gas_used: frame.gas_used.to_string(), // Convert from Uint<256, 4> to String for proper serialization
            input: frame.input,
            output: frame.output,
            error: frame.error,
        };

        let trace_data = match chain_info.schema {
            Schema::Ethereum => RpcTraceData::Ethereum(EthereumRpcTraceData {
                common: common_data,
            }),
            Schema::ZKsync => RpcTraceData::ZKsync(ZKsyncRpcTraceData {
                common: common_data,
            }),
        };

        traces.push(trace_data);

        // Process children with updated trace_address
        for (i, nested_call) in frame.calls.into_iter().enumerate() {
            let mut child_address = trace_address.to_owned();
            child_address.push(i);
            traces.extend(process_frame_with_address(
                nested_call,
                tx_hash,
                chain_info,
                block_number,
                child_address,
            ));
        }

        traces
    }

    // Start with empty trace_address for top-level frame
    process_frame_with_address(frame, tx_hash, chain_info, block_number, Vec::new())
}
