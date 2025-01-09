// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use eyre::Result;

use crate::models::common::ChainId;
use crate::models::common::ParsedData;
use crate::models::indexed::blocks::TransformedBlockData;
pub trait BlockTransformer {
    fn transform_blocks(self) -> Result<Vec<TransformedBlockData>>;
}

// TODO: Confirm I want all these fields
impl BlockTransformer for ParsedData {
    fn transform_blocks(self) -> Result<Vec<TransformedBlockData>> {
        Ok(self
            .header
            .into_iter()
            .map(|header| TransformedBlockData {
                chain_id: self.chain_id,
                hash: header.hash,
                parent_hash: header.parent_hash,
                ommers_hash: header.ommers_hash,
                beneficiary: header.beneficiary,
                state_root: header.state_root,
                transactions_root: header.transactions_root,
                receipts_root: header.receipts_root,
                logs_bloom: header.logs_bloom,
                difficulty: header.difficulty,
                block_number: header.block_number,
                gas_limit: header.gas_limit,
                gas_used: header.gas_used,
                block_time: header.block_time,
                block_date: header.block_date,
                extra_data: header.extra_data,
                mix_hash: header.mix_hash,
                nonce: header.nonce,
                base_fee_per_gas: header.base_fee_per_gas,
                withdrawals_root: header.withdrawals_root,
                blob_gas_used: header.blob_gas_used,
                excess_blob_gas: header.excess_blob_gas,
                parent_beacon_block_root: header.parent_beacon_block_root,
                requests_hash: header.requests_hash,
                target_blobs_per_block: header.target_blobs_per_block,
                total_difficulty: header.total_difficulty,
                size: header.size,
            })
            .collect())
    }
}
