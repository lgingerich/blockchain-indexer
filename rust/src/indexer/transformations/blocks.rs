// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use eyre::Result;

use crate::models::indexed::blocks::TransformedBlockData;
use crate::models::common::ParsedData;

pub trait BlockTransformer {
    fn transform_blocks(self) -> Result<Vec<TransformedBlockData>>;
}

// TODO: Confirm I want all these fields
impl BlockTransformer for ParsedData {
    fn transform_blocks(self) -> Result<Vec<TransformedBlockData>> {
        Ok(vec![TransformedBlockData {
            chain_id: self.transactions[0].chain_id.clone(), // TODO: Remove clone. What happens if their are no transactions in the block? This will be empty.
            hash: self.header.hash,
            parent_hash: self.header.parent_hash,
            ommers_hash: self.header.ommers_hash,
            beneficiary: self.header.beneficiary,
            state_root: self.header.state_root,
            transactions_root: self.header.transactions_root,
            receipts_root: self.header.receipts_root,
            logs_bloom: self.header.logs_bloom,
            difficulty: self.header.difficulty,
            number: self.header.number,
            gas_limit: self.header.gas_limit,
            gas_used: self.header.gas_used,
            block_time: self.header.block_time,
            block_date: self.header.block_date,
            extra_data: self.header.extra_data,
            mix_hash: self.header.mix_hash,
            nonce: self.header.nonce,
            base_fee_per_gas: self.header.base_fee_per_gas,
            withdrawals_root: self.header.withdrawals_root,
            blob_gas_used: self.header.blob_gas_used,
            excess_blob_gas: self.header.excess_blob_gas,
            parent_beacon_block_root: self.header.parent_beacon_block_root,
            requests_hash: self.header.requests_hash,
            target_blobs_per_block: self.header.target_blobs_per_block,
            total_difficulty: self.header.total_difficulty,
            size: self.header.size,
        }])
    }
}