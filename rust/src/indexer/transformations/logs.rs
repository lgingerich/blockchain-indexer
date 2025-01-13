// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use anyhow::Result;

use crate::models::common::ParsedData;
use crate::models::indexed::logs::TransformedLogData;

pub trait LogTransformer {
    fn transform_logs(self) -> Result<Vec<TransformedLogData>>;
}

impl LogTransformer for ParsedData {
    fn transform_logs(self) -> Result<Vec<TransformedLogData>> {
        Ok(self
            .logs
            .into_iter()
            .map(|log| TransformedLogData {
                chain_id: self.chain_id,
                address: log.address,
                topics: log.topics,
                data: log.data,
                block_hash: log.block_hash,
                block_number: log.block_number,
                block_timestamp: log.block_timestamp,
                transaction_hash: log.transaction_hash,
                transaction_index: log.transaction_index,
                log_index: log.log_index,
                removed: log.removed,
            })
            .collect())
    }
}
