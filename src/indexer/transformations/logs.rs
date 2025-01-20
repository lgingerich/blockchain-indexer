use anyhow::Result;

use crate::models::common::{Chain, ParsedData};
use crate::models::datasets::logs::{
    CommonTransformedLogData, EthereumTransformedLogData, RpcLogReceiptData, TransformedLogData,
    ZKsyncTransformedLogData,
};

pub trait LogTransformer {
    fn transform_logs(self, chain: Chain) -> Result<Vec<TransformedLogData>>;
}

impl LogTransformer for ParsedData {
    fn transform_logs(self, chain: Chain) -> Result<Vec<TransformedLogData>> {
        Ok(self
            .logs
            .into_iter()
            .map(|log| {
                // First match on the log to get the common data
                let common_data = match &log {
                    RpcLogReceiptData::Ethereum(l) => &l.common,
                    RpcLogReceiptData::ZKsync(l) => &l.common,
                };

                let common = CommonTransformedLogData {
                    chain_id: self.chain_id,
                    address: common_data.address,
                    topics: common_data.topics.clone(),
                    data: common_data.data.clone(),
                    block_hash: common_data.block_hash,
                    block_number: common_data.block_number,
                    block_timestamp: common_data.block_timestamp,
                    transaction_hash: common_data.transaction_hash,
                    transaction_index: common_data.transaction_index,
                    log_index: common_data.log_index,
                    removed: common_data.removed,
                };

                match chain {
                    Chain::Ethereum => {
                        TransformedLogData::Ethereum(EthereumTransformedLogData { common })
                    }
                    Chain::ZKsync => {
                        TransformedLogData::ZKsync(ZKsyncTransformedLogData { common })
                    }
                }
            })
            .collect())
    }
}
