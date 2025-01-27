use alloy_consensus::constants::{
    EIP1559_TX_TYPE_ID, EIP2930_TX_TYPE_ID, EIP4844_TX_TYPE_ID, EIP7702_TX_TYPE_ID,
    LEGACY_TX_TYPE_ID,
};
use alloy_consensus::{TxEip4844Variant, TxEnvelope};
use alloy_eips::eip2930::AccessList;
use alloy_network::{primitives::BlockTransactions, AnyRpcBlock, AnyTxEnvelope};
use alloy_primitives::{Address, Bytes, FixedBytes, Uint};
use anyhow::Result;
use chrono::DateTime;

use crate::models::common::{Chain, TransactionTo};
use crate::models::datasets::blocks::{
    CommonRpcHeaderData, EthereumRpcHeaderData, RpcHeaderData, ZKsyncRpcHeaderData,
};
use crate::models::datasets::transactions::{
    CommonRpcTransactionData, EthereumRpcTransactionData, RpcTransactionData,
    ZKsyncRpcTransactionData,
};
use crate::models::errors::BlockError;
use crate::utils::hex_to_u64;

pub trait BlockParser {
    fn parse_header(self, chain: Chain) -> Result<Vec<RpcHeaderData>>;
    fn parse_transactions(self, chain: Chain) -> Result<Vec<RpcTransactionData>>;
}

impl BlockParser for AnyRpcBlock {
    fn parse_header(self, chain: Chain) -> Result<Vec<RpcHeaderData>> {
        let inner = self.header.inner.clone();
        let other = self.other.clone();

        // Define common fields that exist across all chains
        let common = CommonRpcHeaderData {
            block_time: DateTime::from_timestamp(inner.timestamp as i64, 0)
                .expect("invalid timestamp"),
            block_date: DateTime::from_timestamp(inner.timestamp as i64, 0)
                .expect("invalid timestamp")
                .date_naive(),
            block_number: inner.number,
            block_hash: self.header.hash,
            parent_hash: inner.parent_hash,
            nonce: inner.nonce,
            gas_limit: inner.gas_limit,
            gas_used: inner.gas_used,
            base_fee_per_gas: inner.base_fee_per_gas,
            blob_gas_used: inner.blob_gas_used,
            excess_blob_gas: inner.excess_blob_gas,
            extra_data: inner.extra_data,
            difficulty: inner.difficulty.to_string(),
            total_difficulty: self.header.total_difficulty.map(|value| value.to_string()),
            size: self.header.size.map(|value| value.to_string()),
            beneficiary: inner.beneficiary,
            mix_hash: inner.mix_hash,
            ommers_hash: inner.ommers_hash,
            requests_hash: inner.requests_hash,
            logs_bloom: inner.logs_bloom,
            parent_beacon_block_root: inner.parent_beacon_block_root,
            receipts_root: inner.receipts_root,
            state_root: inner.state_root,
            transactions_root: inner.transactions_root,
            withdrawals_root: inner.withdrawals_root,
        };

        let header = match chain {
            Chain::Ethereum => RpcHeaderData::Ethereum(EthereumRpcHeaderData { common }),
            Chain::ZKsync => {
                RpcHeaderData::ZKsync(ZKsyncRpcHeaderData {
                    common,
                    target_blobs_per_block: other
                        .get_deserialized::<String>("targetBlobsPerBlock")
                        .and_then(|result| result.ok())
                        .and_then(hex_to_u64),
                    l1_batch_number: other
                        .get_deserialized::<String>("l1BatchNumber")
                        .and_then(|result| result.ok())
                        .and_then(hex_to_u64),
                    l1_batch_timestamp: other
                        .get_deserialized::<String>("l1BatchTimestamp")
                        .and_then(|result| result.ok())
                        .and_then(hex_to_u64)
                        .and_then(|timestamp| DateTime::from_timestamp(timestamp as i64, 0)),
                    // seal_fields: other.get_deserialized::<Vec<String>>("sealFields").and_then(|result| result.ok()), // TODO: Add this back in
                })
            }
        };

        Ok(vec![header])
    }

    fn parse_transactions(self, chain: Chain) -> Result<Vec<RpcTransactionData>> {
        match self.transactions {
            BlockTransactions::Full(_) => {
                Ok(self
                .transactions
                .txns()
                .map(|transaction| {

                    let inner = transaction.inner.clone();
                    let block_hash = transaction.block_hash;
                    let block_number = transaction.block_number;
                    let tx_index = transaction.transaction_index;
                    let effective_gas_price = transaction.effective_gas_price;
                    let from = transaction.from;

                    // default values of mandatory fields are not too important as they will always get overrriden by the actual values
                    let common = CommonRpcTransactionData {
                        block_number,
                        block_hash,
                        tx_hash: FixedBytes::<32>::ZERO,
                        tx_index,
                        tx_type: 0, // Required field. Always overridden by actual value
                        nonce: 0, // Required field. Always overridden by actual value
                        from,
                        to: TransactionTo::Address(Address::ZERO), // Required field. Always overridden by actual value
                        input: None, // Required field. Always overridden by actual value
                        value: None, // Required field. Always overridden by actual value
                        gas_price: None,
                        gas_limit: 0, // Required field. Always overridden by actual value
                        max_fee_per_gas: None,
                        max_priority_fee_per_gas: None,
                        effective_gas_price,
                        access_list: AccessList::default(),
                        authorization_list: Vec::new(),
                        blob_versioned_hashes: Vec::new(),
                        r: None,
                        s: None,
                        v: None,
                    };

                    // TODO: Change to match on chains first.
                    // This current method makes it difficult to get fields nested
                    // under `other` for non-Ethereum chains.
                    // - e.g. l1_batch_number, l1_batch_tx_index, max_fee_per_gas, max_priority_fee_per_gas
                    match &inner.inner {
                        // Ethereum will always enter this match arm
                        // Other chains will only enter this match arm for tx_type = Legacy
                        AnyTxEnvelope::Ethereum(inner) => {
                            let common_tx = match inner {
                                TxEnvelope::Legacy(signed) => {
                                    let tx = signed.tx();
                                    let signature = signed.signature();

                                    RpcTransactionData::Ethereum(EthereumRpcTransactionData {
                                        common: CommonRpcTransactionData {
                                            tx_hash: *signed.hash(),
                                            tx_type: LEGACY_TX_TYPE_ID,
                                            nonce: tx.nonce,
                                            to: TransactionTo::TxKind(tx.to),
                                            input: Some(tx.input.clone()),
                                            value: Some(tx.value.to_string()),
                                            gas_price: Some(tx.gas_price),
                                            gas_limit: tx.gas_limit,
                                            r: Some(signature.r().to_string()),
                                            s: Some(signature.s().to_string()),
                                            v: Some(signature.v()),
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
                                    let signature = signed.signature();

                                    RpcTransactionData::Ethereum(EthereumRpcTransactionData {
                                        common: CommonRpcTransactionData {
                                            tx_hash: *signed.hash(),
                                            tx_type: EIP2930_TX_TYPE_ID,
                                            nonce: tx.nonce,
                                            to: TransactionTo::TxKind(tx.to),
                                            input: Some(tx.input.clone()),
                                            value: Some(tx.value.to_string()),
                                            gas_price: Some(tx.gas_price),
                                            gas_limit: tx.gas_limit,
                                            access_list: tx.access_list.clone(),
                                            r: Some(signature.r().to_string()),
                                            s: Some(signature.s().to_string()),
                                            v: Some(signature.v()),
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
                                    let signature = signed.signature();

                                    RpcTransactionData::Ethereum(EthereumRpcTransactionData {
                                        common: CommonRpcTransactionData {
                                            tx_hash: *signed.hash(),
                                            // tx_type: tx.tx_type(), // TODO: Not publicly accessible. Fix
                                            tx_type: EIP1559_TX_TYPE_ID,
                                            nonce: tx.nonce,
                                            to: TransactionTo::TxKind(tx.to),
                                            input: Some(tx.input.clone()),
                                            value: Some(tx.value.to_string()),
                                            gas_limit: tx.gas_limit,
                                            max_fee_per_gas: Some(tx.max_fee_per_gas),
                                            max_priority_fee_per_gas: Some(tx.max_priority_fee_per_gas),
                                            access_list: tx.access_list.clone(),
                                            r: Some(signature.r().to_string()),
                                            s: Some(signature.s().to_string()),
                                            v: Some(signature.v()),
                                            ..common
                                        },
                                        max_fee_per_blob_gas: None,
                                        blobs: Vec::new(),
                                        commitments: Vec::new(),
                                        proofs: Vec::new(),
                                    })
                                }
                                TxEnvelope::Eip4844(signed) => {
                                    let signature = signed.signature();

                                    match signed.tx() {
                                        TxEip4844Variant::TxEip4844(tx) => RpcTransactionData::Ethereum(EthereumRpcTransactionData {
                                            common: CommonRpcTransactionData {
                                                tx_hash: *signed.hash(),
                                                tx_type: EIP4844_TX_TYPE_ID,
                                                nonce: tx.nonce,
                                                to: TransactionTo::Address(tx.to),
                                                input: Some(tx.input.clone()),
                                                value: Some(tx.value.to_string()),
                                                gas_limit: tx.gas_limit,
                                                max_fee_per_gas: Some(tx.max_fee_per_gas),
                                                max_priority_fee_per_gas: Some(tx.max_priority_fee_per_gas),
                                                access_list: tx.access_list.clone(),
                                                blob_versioned_hashes: tx.blob_versioned_hashes.clone(),
                                                r: Some(signature.r().to_string()),
                                                s: Some(signature.s().to_string()),
                                                v: Some(signature.v()),
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
                                                    to: TransactionTo::Address(tx.to),
                                                    input: Some(tx.input.clone()),
                                                    value: Some(tx.value.to_string()),
                                                    gas_limit: tx.gas_limit,
                                                    max_fee_per_gas: Some(tx.max_fee_per_gas),
                                                    max_priority_fee_per_gas: Some(tx.max_priority_fee_per_gas),
                                                    access_list: tx.access_list.clone(),
                                                    blob_versioned_hashes: tx.blob_versioned_hashes.clone(),
                                                    r: Some(signature.r().to_string()),
                                                    s: Some(signature.s().to_string()),
                                                    v: Some(signature.v()),
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
                                    let signature = signed.signature();

                                    RpcTransactionData::Ethereum(EthereumRpcTransactionData {
                                        common: CommonRpcTransactionData {
                                            tx_hash: *signed.hash(),
                                            tx_type: EIP7702_TX_TYPE_ID,
                                            nonce: tx.nonce,
                                            to: TransactionTo::Address(tx.to),
                                            input: Some(tx.input.clone()),
                                            value: Some(tx.value.to_string()),
                                            gas_limit: tx.gas_limit,
                                            max_fee_per_gas: Some(tx.max_fee_per_gas),
                                            max_priority_fee_per_gas: Some(tx.max_priority_fee_per_gas),
                                            access_list: tx.access_list.clone(),
                                            authorization_list: tx.authorization_list.clone(),
                                            r: Some(signature.r().to_string()),
                                            s: Some(signature.s().to_string()),
                                            v: Some(signature.v()),
                                            ..common
                                        },
                                        max_fee_per_blob_gas: None,
                                        blobs: Vec::new(),
                                        commitments: Vec::new(),
                                        proofs: Vec::new(),
                                    })
                                }
                                _ => RpcTransactionData::Ethereum(EthereumRpcTransactionData {
                                    common,
                                    max_fee_per_blob_gas: None,
                                    blobs: Vec::new(),
                                    commitments: Vec::new(),
                                    proofs: Vec::new(),
                                })
                            };


                            // Non-Ethereum chains will match on AnyTxEnvelope::Ethereum
                            // for legacy transactions. This handles converting back to
                            // proper chain type.
                            let other = transaction.other.clone();
                            match chain {
                                Chain::Ethereum => common_tx,
                                Chain::ZKsync => match common_tx {
                                    RpcTransactionData::Ethereum(t) => {
                                        RpcTransactionData::ZKsync(ZKsyncRpcTransactionData {
                                            common: t.common,
                                            l1_batch_number: other.get_deserialized::<String>("l1BatchNumber")
                                                .and_then(|result| result.ok())
                                                .and_then(hex_to_u64),
                                            l1_batch_tx_index: other.get_deserialized::<String>("l1BatchTxIndex")
                                                .and_then(|result| result.ok())
                                                .and_then(hex_to_u64),
                                        })
                                    },
                                    _ => unreachable!("Expected Ethereum transaction format for legacy transaction") // TODO: Is this ok?
                                }
                            }
                        }
                        // Ethereum should never enter this match arm
                        // Other chains will enter this match arm for tx_type != Legacy
                        // TODO: Handle better
                        AnyTxEnvelope::Unknown(unknown) => {

                            let other_fields = &unknown.inner.fields;
                            let memo = &unknown.inner.memo;
                            let inner = &unknown.inner;
                            let ty = inner.ty;

                            let common_fields = CommonRpcTransactionData {
                                block_number,
                                block_hash,
                                tx_hash: unknown.hash,
                                tx_index,
                                tx_type: ty.0, // Gets the first element of the tuple as u8
                                nonce: other_fields
                                    .get_deserialized::<u64>("nonce")
                                    .and_then(|result| result.ok())
                                    .unwrap_or_default(),
                                from,
                                to: other_fields
                                    .get_deserialized::<TransactionTo>("to")
                                    .and_then(|result| result.ok())
                                    .unwrap_or(TransactionTo::Address(Address::ZERO)),
                                input: other_fields
                                    .get_deserialized::<Bytes>("input")
                                    .and_then(|result| result.ok()),
                                value: other_fields
                                    .get_deserialized::<Uint<256, 4>>("value")
                                    .and_then(|result| result.ok())
                                    .map(|value| value.to_string()),
                                gas_price: Some(other_fields
                                    .get_deserialized::<u128>("gasPrice")
                                    .and_then(|result| result.ok())
                                    .unwrap_or_default()),
                                gas_limit: other_fields
                                    .get_deserialized::<u64>("gas")
                                    .and_then(|result| result.ok())
                                    .unwrap_or_default(),
                                max_fee_per_gas: other_fields
                                    .get_deserialized::<u128>("maxFeePerGas")
                                    .and_then(|result| result.ok()),
                                max_priority_fee_per_gas: other_fields
                                    .get_deserialized::<u128>("maxPriorityFeePerGas")
                                    .and_then(|result| result.ok()),
                                effective_gas_price,
                                access_list: memo.access_list
                                    .get()
                                    .cloned()
                                    .unwrap_or_default(),
                                authorization_list: memo.authorization_list
                                    .get()
                                    .cloned()
                                    .unwrap_or_default(),
                                blob_versioned_hashes: memo.blob_versioned_hashes
                                    .get()
                                    .cloned()
                                    .unwrap_or_default(),
                                r: other_fields
                                    .get_deserialized::<Uint<256, 4>>("r")
                                    .and_then(|result| result.ok())
                                    .map(|r| r.to_string()),
                                s: other_fields
                                    .get_deserialized::<Uint<256, 4>>("s")
                                    .and_then(|result| result.ok())
                                    .map(|s| s.to_string()),
                                v: other_fields
                                    .get_deserialized::<bool>("v")
                                    .and_then(|result| result.ok()), // Deserialized as bool
                            };

                            match chain {
                                Chain::Ethereum => {
                                    unreachable!("Ethereum transactions should be handled by AnyTxEnvelope::Ethereum variant") // TODO: Is this ok?
                                }
                                Chain::ZKsync => {
                                    RpcTransactionData::ZKsync(ZKsyncRpcTransactionData {
                                        common: common_fields,
                                        l1_batch_number: other_fields
                                            .get_deserialized::<String>("l1BatchNumber")
                                            .and_then(|result| result.ok())
                                            .and_then(hex_to_u64),
                                        l1_batch_tx_index: other_fields
                                            .get_deserialized::<String>("l1BatchTxIndex")
                                            .and_then(|result| result.ok())
                                            .and_then(hex_to_u64),
                                    })
                                }
                            }
                        }
                    }
                })
                .collect())
            }
            BlockTransactions::Hashes(_) => {
                Err(BlockError::TransactionHashesOnly.into())
            }
            BlockTransactions::Uncle => {
                Err(BlockError::UncleBlocksNotSupported.into())
            }
        }
    }
}
