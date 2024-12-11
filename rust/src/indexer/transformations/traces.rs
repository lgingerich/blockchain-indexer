// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use eyre::Result;

use crate::models::indexed::traces::TransformedTraceData;
use crate::models::common::ParsedData;

pub trait TraceTransformer {
    fn transform_traces(self) -> Result<Vec<TransformedTraceData>>;
}

impl TraceTransformer for ParsedData {
    // The final traces dataset has no transformation from the rpc trace data
    fn transform_traces(self) -> Result<Vec<TransformedTraceData>> {
        Ok(self.traces)
    }
}