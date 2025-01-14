// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use anyhow::Result;

use crate::models::common::ParsedData;
use crate::models::datasets::transactions::TransformedTransactionData;

pub trait TransactionTransformer {
    fn transform_transactions(self) -> Result<Vec<TransformedTransactionData>>;
}

impl TransactionTransformer for ParsedData {
    fn transform_transactions(self) -> Result<Vec<TransformedTransactionData>> {
        // Zip transactions with their corresponding receipts
        let transactions_with_receipts =
            self.transactions.into_iter().zip(self.transaction_receipts);

        // Map each (transaction, receipt) pair into a TransformedTransactionData
        Ok(transactions_with_receipts
            .map(|(tx, receipt)| {
                TransformedTransactionData {
                    chain_id: self.chain_id,
                    tx_type: tx.tx_type,

                    // Fields from TransactionData
                    nonce: tx.nonce,
                    gas_price: tx.gas_price,
                    gas_limit: tx.gas_limit,
                    max_fee_per_gas: tx.max_fee_per_gas,
                    max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                    value: tx.value,
                    access_list: Some(tx.access_list),
                    // authorization_list: Some(tx.authorization_list), // Convert Vec to Option<Vec>
                    blob_versioned_hashes: tx.blob_versioned_hashes,
                    max_fee_per_blob_gas: tx.max_fee_per_blob_gas,
                    blobs: tx.blobs,
                    commitments: tx.commitments,
                    proofs: tx.proofs,
                    input: tx.input,
                    r: tx.r,
                    s: tx.s,
                    v: tx.v,
                    transaction_hash: receipt.transaction_hash, // Use from receipt as it's non-optional

                    // Fields from TransactionReceiptData
                    status: receipt.status,
                    cumulative_gas_used: receipt.cumulative_gas_used,
                    logs_bloom: receipt.logs_bloom,
                    transaction_index: receipt.transaction_index,
                    block_hash: receipt.block_hash,
                    block_number: receipt.block_number,
                    gas_used: receipt.gas_used,
                    effective_gas_price: receipt.effective_gas_price,
                    blob_gas_used: receipt.blob_gas_used,
                    blob_gas_price: receipt.blob_gas_price,
                    from: receipt.from,
                    to: receipt.to,
                    contract_address: receipt.contract_address,

                    // ZKsync fields
                    l1_batch_number: tx.l1_batch_number,
                    l1_batch_tx_index: tx.l1_batch_tx_index,
                }
            })
            .collect())
    }
}
