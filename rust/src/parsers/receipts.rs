// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use alloy_consensus::Eip658Value;
use alloy_rpc_types_eth::{Log, TransactionReceipt};

use eyre::Result;
use chrono::DateTime;

use crate::types::receipts::{LogReceiptData, TransactionReceiptData};


pub trait ReceiptParser {
    fn parse_transaction_receipts(self) -> Result<Vec<TransactionReceiptData>>;
    fn parse_log_receipts(self) -> Result<Vec<LogReceiptData>>;
}

impl ReceiptParser for Vec<TransactionReceipt> {
    fn parse_transaction_receipts(self) -> Result<Vec<TransactionReceiptData>> {
        self.into_iter()
            .map(|receipt| {
                match receipt.inner.as_receipt_with_bloom() {
                    Some(receipt_with_bloom) => {
                        let status = match receipt_with_bloom.receipt.status {
                            Eip658Value::Eip658(success) => Some(success),
                            Eip658Value::PostState(_) => None,
                        };

                        Ok(TransactionReceiptData {
                            status,
                            cumulative_gas_used: receipt_with_bloom.receipt.cumulative_gas_used,
                            logs_bloom: receipt_with_bloom.logs_bloom,
                            transaction_hash: receipt.transaction_hash,
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
                            authorization_list: receipt.authorization_list,
                        })
                    },
                    None => Err(eyre::eyre!("Receipt missing bloom data")),
                }
            })
            .collect()
    }

    fn parse_log_receipts(self) -> Result<Vec<LogReceiptData>> {
        self.into_iter()
            .flat_map(|receipt| {
                match receipt.inner.as_receipt_with_bloom() {
                    Some(receipt_with_bloom) => {
                        receipt_with_bloom.receipt.logs.clone().into_iter().map(|log| { //TODO: Remove clone
                            Ok(LogReceiptData {
                                address: log.inner.address,
                                topics: log.inner.data.topics().to_vec(),
                                data: log.inner.data.data,
                                block_hash: log.block_hash,
                                block_number: log.block_number,
                                block_timestamp: log.block_timestamp,
                                transaction_hash: log.transaction_hash,
                                transaction_index: log.transaction_index,
                                log_index: log.log_index,
                                removed: log.removed,
                            })
                        }).collect()
                    },
                    None => vec![Err(eyre::eyre!("Receipt missing bloom data"))]
                }
            })
            .collect()
    }
}