use crate::models::common::Chain;
use crate::models::datasets::logs::{
    CommonTransformedLogData, EthereumTransformedLogData, RpcLogReceiptData, TransformedLogData,
    ZKsyncTransformedLogData,
};

use anyhow::Result;

pub trait LogTransformer {
    fn transform_logs(
        logs: Vec<RpcLogReceiptData>,
        chain: Chain,
        chain_id: u64,
    ) -> Result<Vec<TransformedLogData>>;
}

impl LogTransformer for RpcLogReceiptData {
    fn transform_logs(
        logs: Vec<RpcLogReceiptData>,
        chain: Chain,
        chain_id: u64,
    ) -> Result<Vec<TransformedLogData>> {
        logs.into_iter()
            .map(|log| {
                let common_data = match &log {
                    RpcLogReceiptData::Ethereum(l) => &l.common,
                    RpcLogReceiptData::ZKsync(l) => &l.common,
                };

                // Build primary key - require tx_hash and log_index
                let tx_hash = common_data
                    .tx_hash
                    .ok_or_else(|| anyhow::anyhow!("Missing tx_hash for log primary key"))?;
                let log_index = common_data
                    .log_index
                    .ok_or_else(|| anyhow::anyhow!("Missing log_index for log primary key"))?;
                let pk = format!("log_{}_{}_{}", chain_id, tx_hash, log_index);

                let common = CommonTransformedLogData {
                    id: pk,
                    chain_id,
                    block_time: common_data.block_time,
                    block_date: common_data.block_date,
                    block_number: common_data.block_number,
                    block_hash: common_data.block_hash,
                    tx_hash: common_data.tx_hash,
                    tx_index: common_data.tx_index,
                    log_index: common_data.log_index,
                    address: common_data.address,
                    topics: common_data.topics.clone(),
                    data: common_data.data.clone(),
                };

                Ok(match chain {
                    Chain::Ethereum => {
                        TransformedLogData::Ethereum(EthereumTransformedLogData { common })
                    }
                    Chain::ZKsync => {
                        TransformedLogData::ZKsync(ZKsyncTransformedLogData { common })
                    }
                })
            })
            .collect::<Result<Vec<_>>>()
    }
}
