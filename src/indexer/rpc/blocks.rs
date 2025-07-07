use alloy_consensus::{
    TxEip4844Variant, TxEnvelope,
    constants::{
        EIP1559_TX_TYPE_ID, EIP2930_TX_TYPE_ID, EIP4844_TX_TYPE_ID, EIP7702_TX_TYPE_ID,
        LEGACY_TX_TYPE_ID,
    },
};
use alloy_network::{AnyRpcBlock, AnyTxEnvelope, primitives::BlockTransactions};
use alloy_primitives::{Address, Bytes, FixedBytes, TxKind, Uint};
use anyhow::Result;
use chrono::DateTime;

use crate::models::{
    common::{Chain, TransactionTo},
    datasets::{
        blocks::{CommonRpcHeaderData, EthereumRpcHeaderData, RpcHeaderData, ZKsyncRpcHeaderData},
        transactions::{
            CommonRpcTransactionData, EthereumRpcTransactionData, RpcTransactionData,
            ZKsyncRpcTransactionData,
        },
    },
};
use crate::utils::{hex_to_u64, hex_to_u128, sanitize_block_time};

pub trait BlockParser {
    fn parse_header(&self, chain: Chain) -> Result<Vec<RpcHeaderData>>;
    fn parse_transactions(&self, chain: Chain) -> Result<Vec<RpcTransactionData>>;
}

impl BlockParser for AnyRpcBlock {
    fn parse_header(&self, chain: Chain) -> Result<Vec<RpcHeaderData>> {
        let inner = &self.header.inner;
        let other = &self.other;

        // Get the block timestamp and convert to DateTime
        let original_time =
            DateTime::from_timestamp(inner.timestamp as i64, 0).ok_or_else(|| {
                anyhow::anyhow!("Invalid DateTime from timestamp: {}", inner.timestamp)
            })?;

        // Sanitize the block time if it's block 0 with a 1970 date
        let block_time = sanitize_block_time(inner.number, original_time)?;

        // Define common fields that exist across all chains
        let common = CommonRpcHeaderData {
            block_time,
            block_date: block_time.date_naive(),
            block_number: inner.number,
            block_hash: self.header.hash,
            parent_hash: inner.parent_hash,
            nonce: inner.nonce,
            gas_limit: inner.gas_limit,
            gas_used: inner.gas_used,
            base_fee_per_gas: inner.base_fee_per_gas,
            blob_gas_used: inner.blob_gas_used,
            excess_blob_gas: inner.excess_blob_gas,
            extra_data: inner.extra_data.clone(),
            difficulty: inner.difficulty.to_string(),
            total_difficulty: self.header.total_difficulty.map(|value| value.to_string()),
            size: self.header.size.map(|value| value.to_string()),
            miner: inner.beneficiary,
            logs_bloom: inner.logs_bloom,
            sha3_uncles: inner.ommers_hash,
            receipts_root: inner.receipts_root,
            state_root: inner.state_root,
            transactions_root: inner.transactions_root,
            withdrawals_root: inner.withdrawals_root,
        };

        let header = match chain {
            Chain::Ethereum => RpcHeaderData::Ethereum(EthereumRpcHeaderData { common }),
            Chain::ZKsync => RpcHeaderData::ZKsync(ZKsyncRpcHeaderData {
                common,
                l1_batch_number: other
                    .get_deserialized::<String>("l1BatchNumber")
                    .and_then(std::result::Result::ok)
                    .map(hex_to_u64)
                    .transpose()?,
                l1_batch_timestamp: other
                    .get_deserialized::<String>("l1BatchTimestamp")
                    .and_then(std::result::Result::ok)
                    .map(|s: String| {
                        let timestamp = hex_to_u64(s)?;
                        DateTime::from_timestamp(timestamp as i64, 0).ok_or_else(|| {
                            anyhow::anyhow!("Invalid DateTime from timestamp: {}", timestamp)
                        })
                    })
                    .transpose()?,
            }),
        };

        Ok(vec![header])
    }

    fn parse_transactions(&self, chain: Chain) -> Result<Vec<RpcTransactionData>> {
        match &self.transactions {
            BlockTransactions::Full(txs) => {
                Ok(txs
                .iter()
                .map(|transaction| -> Result<RpcTransactionData> {
                    let inner = &transaction.inner;
                    let block_hash = transaction.block_hash;
                    let block_number = transaction.block_number;
                    let tx_index = transaction.transaction_index;
                    let effective_gas_price = transaction.effective_gas_price;
                    let from_address = transaction.other
                        .get_deserialized::<Address>("from")
                        .and_then(std::result::Result::ok)
                        .unwrap_or(Address::ZERO);

                    // default values of mandatory fields are not too important as they will always get overrriden by the actual values
                    // TODO: Can this be improved?
                    let common = CommonRpcTransactionData {
                        block_number,
                        block_hash,
                        tx_hash: FixedBytes::<32>::ZERO,
                        tx_index,
                        tx_type: 0, // Required field. Always overridden by actual value
                        nonce: 0, // Required field. Always overridden by actual value
                        from_address,
                        to_address: TransactionTo::Address(Address::ZERO), // Required field. Always overridden by actual value
                        input: None, // Required field. Always overridden by actual value
                        value: None, // Required field. Always overridden by actual value
                        gas_price: None,
                        gas_limit: 0, // Required field. Always overridden by actual value
                        max_fee_per_gas: None,
                        max_priority_fee_per_gas: None,
                        effective_gas_price,
                        blob_versioned_hashes: Vec::new(),
                    };

                    // TODO: Change to match on chains first.
                    // This current method makes it difficult to get fields nested
                    // under `other` for non-Ethereum chains.
                    // - e.g. l1_batch_number, l1_batch_tx_index, max_fee_per_gas, max_priority_fee_per_gas
                    match &*inner.inner {
                        // Ethereum will always enter this match arm
                        // Other chains will only enter this match arm for tx_type = Legacy
                        AnyTxEnvelope::Ethereum(inner) => {
                            let eth_tx_data = match inner {
                                TxEnvelope::Legacy(signed) => {
                                    let tx = signed.tx();

                                    RpcTransactionData::Ethereum(EthereumRpcTransactionData {
                                        common: CommonRpcTransactionData {
                                            tx_hash: *signed.hash(),
                                            tx_type: LEGACY_TX_TYPE_ID,
                                            nonce: tx.nonce,
                                            to_address: TransactionTo::TxKind(tx.to),
                                            input: Some(tx.input.clone()),
                                            value: Some(tx.value.to_string()),
                                            gas_price: Some(tx.gas_price),
                                            gas_limit: tx.gas_limit,
                                            ..common
                                        },
                                        max_fee_per_blob_gas: None,
                                        blobs: Vec::new(),
                                        commitments: Vec::new(),
                                        proofs: Vec::new(),
                                    })
                                }
                                TxEnvelope::Eip2930(signed) => {
                                    let tx = signed.tx();

                                    RpcTransactionData::Ethereum(EthereumRpcTransactionData {
                                        common: CommonRpcTransactionData {
                                            tx_hash: *signed.hash(),
                                            tx_type: EIP2930_TX_TYPE_ID,
                                            nonce: tx.nonce,
                                            to_address: TransactionTo::TxKind(tx.to),
                                            input: Some(tx.input.clone()),
                                            value: Some(tx.value.to_string()),
                                            gas_price: Some(tx.gas_price),
                                            gas_limit: tx.gas_limit,
                                            ..common
                                        },
                                        max_fee_per_blob_gas: None,
                                        blobs: Vec::new(),
                                        commitments: Vec::new(),
                                        proofs: Vec::new(),
                                    })
                                }
                                TxEnvelope::Eip1559(signed) => {
                                    let tx = signed.tx();

                                    RpcTransactionData::Ethereum(EthereumRpcTransactionData {
                                        common: CommonRpcTransactionData {
                                            tx_hash: *signed.hash(),
                                            // tx_type: tx.tx_type(), // TODO: Not publicly accessible. Fix
                                            tx_type: EIP1559_TX_TYPE_ID,
                                            nonce: tx.nonce,
                                            to_address: TransactionTo::TxKind(tx.to),
                                            input: Some(tx.input.clone()),
                                            value: Some(tx.value.to_string()),
                                            gas_limit: tx.gas_limit,
                                            max_fee_per_gas: Some(tx.max_fee_per_gas),
                                            max_priority_fee_per_gas: Some(tx.max_priority_fee_per_gas),
                                            ..common
                                        },
                                        max_fee_per_blob_gas: None,
                                        blobs: Vec::new(),
                                        commitments: Vec::new(),
                                        proofs: Vec::new(),
                                    })
                                }
                                TxEnvelope::Eip4844(signed) => {

                                    match signed.tx() {
                                        TxEip4844Variant::TxEip4844(tx) => RpcTransactionData::Ethereum(EthereumRpcTransactionData {
                                            common: CommonRpcTransactionData {
                                                tx_hash: *signed.hash(),
                                                tx_type: EIP4844_TX_TYPE_ID,
                                                nonce: tx.nonce,
                                                to_address: TransactionTo::Address(tx.to),
                                                input: Some(tx.input.clone()),
                                                value: Some(tx.value.to_string()),
                                                gas_limit: tx.gas_limit,
                                                max_fee_per_gas: Some(tx.max_fee_per_gas),
                                                max_priority_fee_per_gas: Some(tx.max_priority_fee_per_gas),
                                                blob_versioned_hashes: tx.blob_versioned_hashes.clone(),
                                                ..common
                                            },
                                            max_fee_per_blob_gas: Some(tx.max_fee_per_blob_gas),
                                            blobs: Vec::new(),
                                            commitments: Vec::new(),
                                            proofs: Vec::new(),
                                        }),
                                        TxEip4844Variant::TxEip4844WithSidecar(tx_with_sidecar) => {
                                            let tx = &tx_with_sidecar.tx;

                                            RpcTransactionData::Ethereum(EthereumRpcTransactionData {
                                                common: CommonRpcTransactionData {
                                                    tx_hash: *signed.hash(),
                                                    tx_type: EIP4844_TX_TYPE_ID,
                                                    nonce: tx.nonce,
                                                    to_address: TransactionTo::Address(tx.to),
                                                    input: Some(tx.input.clone()),
                                                    value: Some(tx.value.to_string()),
                                                    gas_limit: tx.gas_limit,
                                                    max_fee_per_gas: Some(tx.max_fee_per_gas),
                                                    max_priority_fee_per_gas: Some(tx.max_priority_fee_per_gas),
                                                    blob_versioned_hashes: tx.blob_versioned_hashes.clone(),
                                                    ..common
                                                },
                                                max_fee_per_blob_gas: Some(tx.max_fee_per_blob_gas),
                                                blobs: tx_with_sidecar.sidecar.blobs.clone(),
                                                commitments: tx_with_sidecar.sidecar.commitments.clone(),
                                                proofs: tx_with_sidecar.sidecar.proofs.clone(),
                                            })
                                        }
                                    }
                                }
                                TxEnvelope::Eip7702(signed) => {
                                    let tx = signed.tx();

                                    RpcTransactionData::Ethereum(EthereumRpcTransactionData {
                                        common: CommonRpcTransactionData {
                                            tx_hash: *signed.hash(),
                                            tx_type: EIP7702_TX_TYPE_ID,
                                            nonce: tx.nonce,
                                            to_address: TransactionTo::Address(tx.to),
                                            input: Some(tx.input.clone()),
                                            value: Some(tx.value.to_string()),
                                            gas_limit: tx.gas_limit,
                                            max_fee_per_gas: Some(tx.max_fee_per_gas),
                                            max_priority_fee_per_gas: Some(tx.max_priority_fee_per_gas),
                                            ..common
                                        },
                                        max_fee_per_blob_gas: None,
                                        blobs: Vec::new(),
                                        commitments: Vec::new(),
                                        proofs: Vec::new(),
                                    })
                                }
                            };


                            // Non-Ethereum chains will match on AnyTxEnvelope::Ethereum
                            // for legacy transactions. This handles converting back to
                            // proper chain type.
                            let other = &transaction.other;
                            Ok(match chain {
                                Chain::Ethereum => eth_tx_data,
                                Chain::ZKsync => match eth_tx_data {
                                    RpcTransactionData::Ethereum(t) => {
                                        RpcTransactionData::ZKsync(ZKsyncRpcTransactionData {
                                            common: CommonRpcTransactionData {
                                                max_fee_per_gas: other
                                                    .get_deserialized::<String>("maxFeePerGas")
                                                    .and_then(std::result::Result::ok)
                                                    .map(hex_to_u128)
                                                    .transpose()?,
                                                max_priority_fee_per_gas: other
                                                    .get_deserialized::<String>("maxPriorityFeePerGas")
                                                    .and_then(std::result::Result::ok)
                                                    .map(hex_to_u128)
                                                    .transpose()?,
                                                ..t.common
                                            },
                                            l1_batch_number: other
                                                .get_deserialized::<String>("l1BatchNumber")
                                                .and_then(std::result::Result::ok)
                                                .map(hex_to_u64)
                                                .transpose()?,
                                            l1_batch_tx_index: other
                                                .get_deserialized::<String>("l1BatchTxIndex")
                                                .and_then(std::result::Result::ok)
                                                .map(hex_to_u64)
                                                .transpose()?,
                                        })
                                    },
                                    _ => unreachable!("Expected Ethereum transaction format for legacy transaction")
                                }
                            })
                        }
                        // Ethereum should never enter this match arm
                        // Other chains will enter this match arm for tx_type != Legacy
                        // TODO: Handle better
                        AnyTxEnvelope::Unknown(unknown) => {
                            let other_fields = &unknown.inner.fields;
                            let memo = &unknown.inner.memo;
                            let inner = &unknown.inner;
                            let ty = inner.ty;

                            let nonce_str = other_fields
                                .get_deserialized::<String>("nonce")
                                .and_then(std::result::Result::ok)
                                .ok_or_else(|| anyhow::anyhow!("'nonce' field missing"))?;
                            let nonce = hex_to_u64(nonce_str)?;

                            let gas_str = other_fields
                                .get_deserialized::<String>("gas")
                                .and_then(std::result::Result::ok)
                                .ok_or_else(|| anyhow::anyhow!("'gas' field missing"))?;
                            let gas_limit = hex_to_u64(gas_str)?;

                            let to_address = other_fields
                                .get_deserialized::<TxKind>("to")
                                .transpose()
                                .map_err(|e| anyhow::anyhow!("failed to deserialize 'to' as TxKind: {}", e))?
                                .map(TransactionTo::TxKind)
                                .or_else(|| {
                                    // If TxKind parsing fails, try parsing as Address
                                    other_fields
                                        .get_deserialized::<Address>("to")
                                        .transpose() // Convert Option<Result<Address, _>> to Result<Option<Address>, _>
                                        .map_err(|e| anyhow::anyhow!("failed to deserialize 'to' as Address: {}", e))
                                        .ok() // Convert Result to Option, discarding errors
                                        .flatten() // Flatten Option<Option<Address>> to Option<Address>
                                        .map(TransactionTo::Address)
                                })
                                .ok_or_else(|| anyhow::anyhow!("failed to deserialize 'to' as either TxKind or Address, or field missing"))?;


                            let common_fields = CommonRpcTransactionData {
                                block_number,
                                block_hash,
                                tx_hash: unknown.hash,
                                tx_index,
                                tx_type: ty.0, // Gets the first element of the tuple as u8
                                nonce,
                                from_address,
                                to_address,
                                input: other_fields
                                    .get_deserialized::<Bytes>("input")
                                    .and_then(std::result::Result::ok),
                                value: other_fields
                                    .get_deserialized::<Uint<256, 4>>("value")
                                    .and_then(std::result::Result::ok)
                                    .map(|value| value.to_string()),
                                gas_price: other_fields
                                    .get_deserialized::<String>("gasPrice")
                                    .and_then(std::result::Result::ok)
                                    .map(hex_to_u128)
                                    .transpose()?,
                                gas_limit,
                                max_fee_per_gas: other_fields
                                    .get_deserialized::<String>("maxFeePerGas")
                                    .and_then(std::result::Result::ok)
                                    .map(hex_to_u128)
                                    .transpose()?,
                                max_priority_fee_per_gas: other_fields
                                    .get_deserialized::<String>("maxPriorityFeePerGas")
                                    .and_then(std::result::Result::ok)
                                    .map(hex_to_u128)
                                    .transpose()?,
                                effective_gas_price,
                                blob_versioned_hashes: memo.blob_versioned_hashes
                                    .get()
                                    .map(|a| a.to_owned())
                                    .unwrap_or_default(),
                            };

                            match chain {
                                Chain::Ethereum => {
                                    unreachable!("Ethereum transactions should be handled by AnyTxEnvelope::Ethereum variant") // TODO: Should be able to get rid of this after refactor
                                }
                                Chain::ZKsync => {
                                    Ok(RpcTransactionData::ZKsync(ZKsyncRpcTransactionData {
                                        common: common_fields,
                                        l1_batch_number: other_fields
                                            .get_deserialized::<String>("l1BatchNumber")
                                            .and_then(std::result::Result::ok)
                                            .map(hex_to_u64)
                                            .transpose()?,
                                        l1_batch_tx_index: other_fields
                                            .get_deserialized::<String>("l1BatchTxIndex")
                                            .and_then(std::result::Result::ok)
                                            .map(hex_to_u64)
                                            .transpose()?,
                                    }))
                                }
                            }
                        }
                    }
                })
                .collect::<Result<Vec<_>, _>>()?)
            }
            BlockTransactions::Hashes(_) => Err(anyhow::anyhow!(
                "Transaction hashes only blocks are not supported"
            )),
            BlockTransactions::Uncle => Err(anyhow::anyhow!("Uncle blocks are not supported")),
        }
    }
}
