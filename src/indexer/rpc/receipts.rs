use alloy_consensus::Eip658Value;
use alloy_network::AnyTransactionReceipt;
use anyhow::Result;
use chrono::DateTime;

use crate::models::common::Chain;
use crate::models::datasets::logs::{
    CommonRpcLogReceiptData, EthereumRpcLogReceiptData, RpcLogReceiptData, ZKsyncRpcLogReceiptData,
};
use crate::models::datasets::transactions::{
    CommonRpcTransactionReceiptData, EthereumRpcTransactionReceiptData, RpcTransactionReceiptData,
    ZKsyncRpcTransactionReceiptData,
};
use crate::utils::hex_to_u64;

pub trait ReceiptParser {
    fn parse_transaction_receipts(self, chain: Chain) -> Result<Vec<RpcTransactionReceiptData>>;
    fn parse_log_receipts(self, chain: Chain) -> Result<Vec<RpcLogReceiptData>>;
}

impl ReceiptParser for Vec<AnyTransactionReceipt> {
    fn parse_transaction_receipts(self, chain: Chain) -> Result<Vec<RpcTransactionReceiptData>> {
        self.into_iter()
            .map(|receipt| {
                // Access the inner ReceiptWithBloom through the AnyReceiptEnvelope
                let receipt_with_bloom = &receipt.inner.inner.inner;

                let status = match receipt_with_bloom.receipt.status {
                    Eip658Value::Eip658(success) => Some(success),
                    Eip658Value::PostState(_) => None,
                };

                let common = CommonRpcTransactionReceiptData {
                    block_number: receipt.inner.block_number,
                    block_hash: receipt.inner.block_hash,
                    tx_hash: receipt.inner.transaction_hash,
                    tx_index: receipt.inner.transaction_index,
                    tx_type: receipt.inner.inner.r#type,
                    status,
                    from: receipt.inner.from,
                    to: receipt.inner.to,
                    contract_address: receipt.inner.contract_address,
                    gas_used: receipt.inner.gas_used,
                    effective_gas_price: receipt.inner.effective_gas_price,
                    cumulative_gas_used: receipt_with_bloom.receipt.cumulative_gas_used,
                    blob_gas_price: receipt.inner.blob_gas_price,
                    blob_gas_used: receipt.inner.blob_gas_used,
                    authorization_list: receipt.inner.authorization_list.unwrap_or_default(),
                    logs_bloom: receipt_with_bloom.logs_bloom,
                };

                let receipt = match chain {
                    Chain::Ethereum => {
                        RpcTransactionReceiptData::Ethereum(EthereumRpcTransactionReceiptData {
                            common,
                        })
                    }
                    Chain::ZKsync => {
                        let l1_batch_number = receipt
                            .other
                            .get_deserialized::<String>("l1BatchNumber")
                            .and_then(|result| result.ok())
                            .and_then(hex_to_u64);

                        let l1_batch_tx_index = receipt
                            .other
                            .get_deserialized::<String>("l1BatchTxIndex")
                            .and_then(|result| result.ok())
                            .and_then(hex_to_u64);

                        RpcTransactionReceiptData::ZKsync(ZKsyncRpcTransactionReceiptData {
                            common,
                            l1_batch_number: l1_batch_number,
                            l1_batch_tx_index: l1_batch_tx_index,
                        })
                    }
                };
                Ok(receipt)
            })
            .collect()
    }

    fn parse_log_receipts(self, chain: Chain) -> Result<Vec<RpcLogReceiptData>> {
        self.into_iter()
            .flat_map(|receipt| {
                let receipt_with_bloom = &receipt.inner.inner.inner;
                receipt_with_bloom
                    .receipt
                    .logs
                    .clone()
                    .into_iter()
                    .map(|log| {
                        let common = CommonRpcLogReceiptData {
                            block_time: log
                                .block_timestamp
                                .and_then(|ts| DateTime::from_timestamp(ts as i64, 0)),
                            block_date: log
                                .block_timestamp
                                .and_then(|ts| DateTime::from_timestamp(ts as i64, 0))
                                .map(|dt| dt.date_naive()),
                            block_number: log.block_number,
                            block_hash: log.block_hash,
                            tx_hash: log.transaction_hash,
                            tx_index: log.transaction_index,
                            log_index: log.log_index,
                            address: log.inner.address,
                            topics: log.inner.data.topics().to_vec(),
                            data: log.inner.data.data,
                            removed: log.removed,
                        };

                        let log = match chain {
                            Chain::Ethereum => {
                                RpcLogReceiptData::Ethereum(EthereumRpcLogReceiptData { common })
                            }
                            Chain::ZKsync => {
                                RpcLogReceiptData::ZKsync(ZKsyncRpcLogReceiptData { common })
                            }
                        };

                        Ok(log)
                    })
                    .collect::<Vec<Result<RpcLogReceiptData>>>()
            })
            .collect()
    }
}
