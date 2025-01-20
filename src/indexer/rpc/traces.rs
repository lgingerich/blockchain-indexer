use alloy_rpc_types_trace::geth::{CallFrame, GethTrace, TraceResult};
use anyhow::Result;

use crate::models::common::Chain;
use crate::models::datasets::traces::{
    CommonRpcTraceData, EthereumRpcTraceData, RpcTraceData, ZKsyncRpcTraceData,
};

pub trait TraceParser {
    fn parse_traces(self, chain: Chain) -> Result<Vec<RpcTraceData>>;
}

impl TraceParser for Vec<TraceResult> {
    fn parse_traces(self, chain: Chain) -> Result<Vec<RpcTraceData>> {
        Ok(self
            .into_iter()
            .flat_map(|trace_result| {
                match trace_result {
                    TraceResult::Success { result, tx_hash: _ } => {
                        match result {
                            GethTrace::CallTracer(frame) => {
                                // Process the frame and all its nested calls
                                flatten_call_frames(frame, chain)
                            }
                            _ => Vec::new(), // Skip other trace types
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
                        Vec::new()
                    }
                }
            })
            .collect())
    }
}

/// Recursively flattens a CallFrame and its nested calls into a vector of TraceData
fn flatten_call_frames(frame: CallFrame, chain: Chain) -> Vec<RpcTraceData> {
    let mut traces = Vec::new();

    let common_data = CommonRpcTraceData {
        from: frame.from,
        gas: frame.gas,
        gas_used: frame.gas_used,
        to: frame.to,
        input: frame.input,
        output: frame.output,
        error: frame.error,
        revert_reason: frame.revert_reason,
        logs: frame.logs,
        value: frame.value,
        typ: frame.typ,
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
        traces.extend(flatten_call_frames(nested_call, chain));
    }

    traces
}
