use anyhow::Result;

use crate::models::{
    common::Chain,
    datasets::blocks::{
        CommonTransformedBlockData, EthereumTransformedBlockData, RpcHeaderData,
        TransformedBlockData, ZKsyncTransformedBlockData,
    },
};

pub trait BlockTransformer {
    fn transform_blocks(
        headers: Vec<RpcHeaderData>,
        chain_info: &ChainInfo,
    ) -> Result<Vec<TransformedBlockData>>;
}

impl BlockTransformer for RpcHeaderData {
    fn transform_blocks(
        headers: Vec<RpcHeaderData>,
        chain_info: &ChainInfo,
    ) -> Result<Vec<TransformedBlockData>> {
        headers
            .into_iter()
            .map(|header| {
                let common_data = match &header {
                    RpcHeaderData::Ethereum(h) => &h.common,
                    RpcHeaderData::ZKsync(h) => &h.common,
                };

                let pk = format!("block_{}_{}", chain_info.id, common_data.block_hash); // Build primary key

                let common = CommonTransformedBlockData {
                    id: pk,
                    chain_id: chain_info.id,
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
                    miner: common_data.miner,
                    logs_bloom: common_data.logs_bloom,
                    sha3_uncles: common_data.sha3_uncles,
                    receipts_root: common_data.receipts_root,
                    state_root: common_data.state_root,
                    transactions_root: common_data.transactions_root,
                    withdrawals_root: common_data.withdrawals_root,
                };

                match chain_info.schema {
                    Schema::Ethereum => Ok(TransformedBlockData::Ethereum(
                        EthereumTransformedBlockData { common },
                    )),
                    Schema::ZKsync => {
                        let zksync_data = match header {
                            RpcHeaderData::ZKsync(data) => data,
                            _ => {
                                return Err(anyhow::anyhow!(
                                    "Expected ZKsync header for ZKsync chain"
                                ));
                            }
                        };

                        Ok(TransformedBlockData::ZKsync(ZKsyncTransformedBlockData {
                            common,
                            l1_batch_number: zksync_data.l1_batch_number,
                            l1_batch_timestamp: zksync_data.l1_batch_timestamp,
                        }))
                    }
                }
            })
            .collect::<Result<Vec<_>>>()
    }
}
