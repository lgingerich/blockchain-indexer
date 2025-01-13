// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use anyhow::Result;

use crate::models::common::ParsedData;
use crate::models::indexed::traces::TransformedTraceData;

pub trait TraceTransformer {
    fn transform_traces(self) -> Result<Vec<TransformedTraceData>>;
}

impl TraceTransformer for ParsedData {
    fn transform_traces(self) -> Result<Vec<TransformedTraceData>> {
        Ok(self
            .traces
            .into_iter()
            .map(|trace| TransformedTraceData {
                chain_id: self.chain_id,
                from: trace.from,
                gas: trace.gas,
                gas_used: trace.gas_used,
                to: trace.to,
                input: trace.input,
                output: trace.output,
                error: trace.error,
                revert_reason: trace.revert_reason,
                logs: trace.logs,
                value: trace.value,
                typ: trace.typ,
            })
            .collect())
    }
}
