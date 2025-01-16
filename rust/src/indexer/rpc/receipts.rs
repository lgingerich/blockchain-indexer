// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use alloy_consensus::Eip658Value;
use alloy_rpc_types_eth::{Log, TransactionReceipt};

use anyhow::{anyhow, Result};
use chrono::DateTime;

use crate::models::datasets::logs::{
    CommonRpcLogReceiptData, RpcLogReceiptData,
    EthereumRpcLogReceiptData, ZKsyncRpcLogReceiptData
};
use crate::models::datasets::transactions::{
    CommonRpcTransactionReceiptData, RpcTransactionReceiptData,
    EthereumRpcTransactionReceiptData, ZKsyncRpcTransactionReceiptData
};
use crate::utils::hex_to_u64;
use crate::models::common::Chain;

use alloy_network::AnyTransactionReceipt;

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
                    transaction_hash: receipt.inner.transaction_hash,
                    transaction_index: receipt.inner.transaction_index,
                    status,
                    tx_type: receipt.inner.inner.r#type,
                    block_hash: receipt.inner.block_hash,
                    block_number: receipt.inner.block_number,
                    gas_used: receipt.inner.gas_used,
                    effective_gas_price: receipt.inner.effective_gas_price,
                    blob_gas_used: receipt.inner.blob_gas_used,
                    blob_gas_price: receipt.inner.blob_gas_price,
                    from: receipt.inner.from,
                    to: receipt.inner.to,
                    contract_address: receipt.inner.contract_address,
                    cumulative_gas_used: receipt_with_bloom.receipt.cumulative_gas_used,
                    logs_bloom: receipt_with_bloom.logs_bloom,
                    authorization_list: receipt.inner.authorization_list,
                };

                let receipt = match chain {
                    Chain::Ethereum => {
                        RpcTransactionReceiptData::Ethereum(EthereumRpcTransactionReceiptData { common })
                    }
                    Chain::ZKsync => {
                        RpcTransactionReceiptData::ZKsync(ZKsyncRpcTransactionReceiptData { 
                            common,
                            l1_batch_number: receipt.other
                                .get_deserialized::<String>("l1BatchNumber")
                                .and_then(|result| result.ok())
                                .and_then(hex_to_u64),
                            l1_batch_tx_index: receipt.other
                                .get_deserialized::<String>("l1BatchTxIndex")
                                .and_then(|result| result.ok())
                                .and_then(hex_to_u64),
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
