use anyhow::Result;
use chrono::{DateTime, Utc, NaiveDate};
use std::collections::HashMap;

use crate::models::common::{Chain, ParsedData};
use crate::models::datasets::transactions::{
    CommonTransformedTransactionData, EthereumTransformedTransactionData, RpcTransactionData,
    RpcTransactionReceiptData, TransformedTransactionData, ZKsyncTransformedTransactionData,
};

pub trait TransactionTransformer {
    fn transform_transactions(self, chain: Chain, block_map: HashMap<u64, (DateTime<Utc>, NaiveDate)>) -> Result<Vec<TransformedTransactionData>>;
}

impl TransactionTransformer for ParsedData {
    fn transform_transactions(self, chain: Chain, block_map: HashMap<u64, (DateTime<Utc>, NaiveDate)>) -> Result<Vec<TransformedTransactionData>> {
        // Zip transactions with their corresponding receipts
        let transactions_with_receipts =
            self.transactions.into_iter().zip(self.transaction_receipts);

        // Map each (transaction, receipt) pair into a TransformedTransactionData
        Ok(transactions_with_receipts
            .map(|(tx, receipt)| {
                // First match on the tx to get the common data
                let common_tx = match &tx {
                    RpcTransactionData::Ethereum(t) => &t.common,
                    RpcTransactionData::ZKsync(t) => &t.common,
                };
                // Then match on the receipt to get the common data
                let common_receipt = match &receipt {
                    RpcTransactionReceiptData::Ethereum(r) => &r.common,
                    RpcTransactionReceiptData::ZKsync(r) => &r.common,
                };

                let common = CommonTransformedTransactionData {
                    chain_id: self.chain_id,
                    block_time: common_tx.block_number
                        .and_then(|num| block_map.get(&num))
                        .map(|(time, _)| *time)
                        .unwrap_or_default(),
                    block_date: common_tx.block_number
                        .and_then(|num| block_map.get(&num))
                        .map(|(_, date)| *date)
                        .unwrap_or_default(),
                    block_number: common_receipt.block_number,
                    block_hash: common_receipt.block_hash,
                    tx_hash: common_receipt.tx_hash,
                    tx_index: common_receipt.tx_index,
                    // TODO: Improve this
                    // Use receipt fields if available, otherwise use transaction fields or defaults
                    // 0 is used as the default value for tx_type if it does not exist, so
                    // if one field is not 0, it means we should use that field
                    tx_type: if common_receipt.tx_type != 0 {
                        common_receipt.tx_type
                    } else {
                        common_tx.tx_type
                    },
                    status: common_receipt.status,
                    nonce: common_tx.nonce,
                    from: common_receipt.from,
                    to: common_receipt.to,
                    contract_address: common_receipt.contract_address,
                    input: common_tx.input.clone(),
                    value: common_tx.value.clone(),
                    gas_price: common_tx.gas_price,
                    gas_limit: common_tx.gas_limit,
                    gas_used: common_receipt.gas_used,
                    max_fee_per_gas: common_tx.max_fee_per_gas,
                    max_priority_fee_per_gas: common_tx.max_priority_fee_per_gas,
                    effective_gas_price: common_receipt.effective_gas_price,
                    cumulative_gas_used: common_receipt.cumulative_gas_used,
                    blob_gas_price: common_receipt.blob_gas_price,
                    blob_gas_used: common_receipt.blob_gas_used,
                    access_list: common_tx.access_list.clone(),
                    authorization_list: common_receipt.authorization_list.clone(),
                    blob_versioned_hashes: common_tx.blob_versioned_hashes.clone(),
                    logs_bloom: common_receipt.logs_bloom,
                    r: common_tx.r.clone(),
                    s: common_tx.s.clone(),
                    v: common_tx.v,
                };

                match chain {
                    Chain::Ethereum => {
                        let eth_tx = match tx {
                            RpcTransactionData::Ethereum(t) => t,
                            _ => panic!("Expected Ethereum transaction for Ethereum chain"),
                        };

                        TransformedTransactionData::Ethereum(EthereumTransformedTransactionData {
                            common,
                            max_fee_per_blob_gas: eth_tx.max_fee_per_blob_gas,
                            blobs: eth_tx.blobs,
                            commitments: eth_tx.commitments,
                            proofs: eth_tx.proofs,
                        })
                    }
                    Chain::ZKsync => {
                        let zksync_tx = match tx {
                            RpcTransactionData::ZKsync(t) => t,
                            _ => panic!("Expected ZKsync transaction for ZKsync chain"),
                        };

                        TransformedTransactionData::ZKsync(ZKsyncTransformedTransactionData {
                            common,
                            l1_batch_number: zksync_tx.l1_batch_number,
                            l1_batch_tx_index: zksync_tx.l1_batch_tx_index,
                        })
                    }
                }
            })
            .collect())
    }
}
