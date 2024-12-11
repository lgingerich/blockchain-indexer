// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use eyre::Result;

use crate::models::indexed::logs::TransformedLogData;
use crate::models::common::ParsedData;

pub trait LogTransformer {
    fn transform_logs(self) -> Result<Vec<TransformedLogData>>;
}

impl LogTransformer for ParsedData {
    // The final logs dataset has no transformation from the rpc receipt data
    fn transform_logs(self) -> Result<Vec<TransformedLogData>> {
        Ok(self.logs)
    }
}