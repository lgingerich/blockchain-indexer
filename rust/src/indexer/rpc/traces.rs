// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use alloy_consensus::{TxEnvelope, TxEip4844Variant};
use alloy_eips::eip2930::AccessList;
use alloy_eips::eip7702::SignedAuthorization;
use alloy_network::primitives::BlockTransactions;
use alloy_primitives::{Address, Bytes, FixedBytes, Uint};
use alloy_rpc_types_eth::{Block, Header, Withdrawals};
use alloy_rpc_types_trace::geth::{CallFrame, GethTrace, TraceResult};

use eyre::Result;
use chrono::DateTime;

use crate::models::rpc::traces::TraceData;

pub trait TraceParser {
    fn parse_traces(self) -> Result<Vec<TraceData>>;
}

impl TraceParser for Vec<TraceResult> {
    fn parse_traces(self) -> Result<Vec<TraceData>> {
        Ok(self.into_iter()
            .flat_map(|trace_result| {
                match trace_result {
                    TraceResult::Success { result, tx_hash } => {
                        match result {
                            GethTrace::CallTracer(frame) => {
                                // Process the frame and all its nested calls
                                flatten_call_frames(frame)
                            },
                            _ => Vec::new(), // Skip other trace types
                        }
                    },
                    TraceResult::Error { error, tx_hash } => Vec::new(), // Skip failed traces
                }
            })
            .collect())
    }
}

/// Recursively flattens a CallFrame and its nested calls into a vector of TraceData
fn flatten_call_frames(frame: CallFrame) -> Vec<TraceData> {
    let mut traces = Vec::new();
    
    // Add the current frame
    traces.push(TraceData {
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
    });

    // Recursively process nested calls
    for nested_call in frame.calls {
        traces.extend(flatten_call_frames(nested_call));
    }

    traces
}