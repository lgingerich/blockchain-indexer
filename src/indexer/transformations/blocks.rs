use anyhow::Result;

use crate::models::common::{Chain, ParsedData};
use crate::models::datasets::blocks::{
    CommonTransformedBlockData, EthereumTransformedBlockData, RpcHeaderData, TransformedBlockData,
    ZKsyncTransformedBlockData,
};

pub trait BlockTransformer {
    fn transform_blocks(self, chain: Chain) -> Result<Vec<TransformedBlockData>>;
}

impl BlockTransformer for ParsedData {
    fn transform_blocks(self, chain: Chain) -> Result<Vec<TransformedBlockData>> {
        Ok(self
            .header
            .into_iter()
            .map(|header| {
                // First match on the header to get the common data
                let common_data = match &header {
                    RpcHeaderData::Ethereum(h) => &h.common,
                    RpcHeaderData::ZKsync(h) => &h.common,
                };

                let common = CommonTransformedBlockData {
                    chain_id: self.chain_id,
                    block_time: common_data.block_time,
                    block_date: common_data.block_date,
                    block_number: common_data.block_number,
                    block_hash: common_data.block_hash,
                    parent_hash: common_data.parent_hash,
                    nonce: common_data.nonce,
                    gas_limit: common_data.gas_limit,
                    gas_used: common_data.gas_used,
                    base_fee_per_gas: common_data.base_fee_per_gas,
                    blob_gas_used: common_data.blob_gas_used,
                    excess_blob_gas: common_data.excess_blob_gas,
                    extra_data: common_data.extra_data.clone(),
                    difficulty: common_data.difficulty.clone(),
                    total_difficulty: common_data.total_difficulty.clone(),
                    size: common_data.size.clone(),                    
                    beneficiary: common_data.beneficiary,
                    mix_hash: common_data.mix_hash,
                    ommers_hash: common_data.ommers_hash,
                    requests_hash: common_data.requests_hash,
                    logs_bloom: common_data.logs_bloom,
                    parent_beacon_block_root: common_data.parent_beacon_block_root,
                    receipts_root: common_data.receipts_root,
                    state_root: common_data.state_root,
                    transactions_root: common_data.transactions_root,
                    withdrawals_root: common_data.withdrawals_root,
                };

                match chain {
                    Chain::Ethereum => {
                        TransformedBlockData::Ethereum(EthereumTransformedBlockData { common })
                    }
                    Chain::ZKsync => {
                        let zksync_data = match header {
                            RpcHeaderData::ZKsync(h) => h,
                            _ => panic!("Expected ZKsync header for ZKsync chain"),
                        };

                        TransformedBlockData::ZKsync(ZKsyncTransformedBlockData {
                            common,
                            target_blobs_per_block: zksync_data.target_blobs_per_block,
                            l1_batch_number: zksync_data.l1_batch_number,
                            l1_batch_timestamp: zksync_data.l1_batch_timestamp,
                        })
                    }
                }
            })
            .collect())
    }
}
