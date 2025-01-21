use alloy_rpc_types_trace::geth::{CallFrame, GethTrace, TraceResult};
use alloy_primitives::FixedBytes;
use anyhow::Result;

use crate::models::common::Chain;
use crate::models::datasets::traces::{
    CommonRpcTraceData, EthereumRpcTraceData, RpcTraceData, ZKsyncRpcTraceData,
};

pub trait TraceParser {
    fn parse_traces(self, chain: Chain, block_number: u64) -> Result<Vec<RpcTraceData>>;
}

impl TraceParser for Vec<TraceResult> {
    fn parse_traces(self, chain: Chain, block_number: u64) -> Result<Vec<RpcTraceData>> {
        Ok(self
            .into_iter()
            .flat_map(|trace_result| {
                match trace_result {
                    TraceResult::Success { result, tx_hash } => {
                        match result {
                            GethTrace::CallTracer(frame) => {
                                // Process the frame and all its nested calls
                                flatten_call_frames(frame, tx_hash, chain, block_number)
                            }
                            _ => Vec::new(), // Skip other trace types
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

/// Recursively flattens a CallFrame and its nested calls into a vector of TraceData
fn flatten_call_frames(
    frame: CallFrame,
    tx_hash: Option<FixedBytes<32>>,
    chain: Chain,
    block_number: u64,
) -> Vec<RpcTraceData> {
    let mut traces = Vec::new();

    let common_data = CommonRpcTraceData {
        block_number,
        tx_hash,
        r#type: frame.typ,
        from: frame.from,
        to: frame.to,
        value: frame.value.map(|v| v.to_string()), // Convert from Uint<256, 4> to String for proper serialization
        gas: frame.gas.to_string(), // Convert from Uint<256, 4> to String for proper serialization
        gas_used: frame.gas_used.to_string(), // Convert from Uint<256, 4> to String for proper serialization
        input: frame.input,
        output: frame.output,
        error: frame.error,
        revert_reason: frame.revert_reason,
        logs: frame.logs,
    };

    let trace_data = match chain {
        Chain::Ethereum => RpcTraceData::Ethereum(EthereumRpcTraceData {
            common: common_data,
        }),
        Chain::ZKsync => RpcTraceData::ZKsync(ZKsyncRpcTraceData {
            common: common_data,
        }),
    };

    // Add the current frame
    traces.push(trace_data);

    // Recursively process nested calls
    for nested_call in frame.calls {
        traces.extend(flatten_call_frames(nested_call, tx_hash, chain, block_number));
    }

    traces
}
