use alloy_consensus::constants::{
    EIP1559_TX_TYPE_ID, EIP2930_TX_TYPE_ID, EIP4844_TX_TYPE_ID, EIP7702_TX_TYPE_ID,
    LEGACY_TX_TYPE_ID,
};
use alloy_consensus::{TxEip4844Variant, TxEnvelope};
use alloy_eips::eip2930::AccessList;
use alloy_network::{primitives::BlockTransactions, AnyRpcBlock, AnyTxEnvelope};
use alloy_primitives::{Address, Bytes, FixedBytes, Uint};
use anyhow::{anyhow, Result};
use chrono::DateTime;

use crate::models::common::{Chain, TransactionTo};
use crate::models::datasets::blocks::{
    CommonRpcHeaderData, EthereumRpcHeaderData, RpcHeaderData, ZKsyncRpcHeaderData,
};
use crate::models::datasets::transactions::{
    CommonRpcTransactionData, EthereumRpcTransactionData, RpcTransactionData,
    ZKsyncRpcTransactionData,
};
use crate::utils::hex_to_u64;

// NOTE: No handling for uncle blocks
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
            hash: self.header.hash,
            parent_hash: inner.parent_hash,
            ommers_hash: inner.ommers_hash,
            beneficiary: inner.beneficiary,
            state_root: inner.state_root,
            transactions_root: inner.transactions_root,
            receipts_root: inner.receipts_root,
            logs_bloom: inner.logs_bloom,
            difficulty: inner.difficulty,
            block_number: inner.number,
            gas_limit: inner.gas_limit,
            gas_used: inner.gas_used,
            block_time: DateTime::from_timestamp(inner.timestamp as i64, 0)
                .expect("invalid timestamp"),
            block_date: DateTime::from_timestamp(inner.timestamp as i64, 0)
                .expect("invalid timestamp")
                .date_naive(),
            extra_data: inner.extra_data,
            mix_hash: inner.mix_hash,
            nonce: inner.nonce,
            base_fee_per_gas: inner.base_fee_per_gas,
            withdrawals_root: inner.withdrawals_root,
            blob_gas_used: inner.blob_gas_used,
            excess_blob_gas: inner.excess_blob_gas,
            parent_beacon_block_root: inner.parent_beacon_block_root,
            requests_hash: inner.requests_hash,
            total_difficulty: self.header.total_difficulty,
            size: self.header.size,
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
                    let transaction_index = transaction.transaction_index;
                    let effective_gas_price = transaction.effective_gas_price;
                    let from = transaction.from;

                    // default values of mandatory fields are not too important as they will always get overrriden by the actual values
                    let common = CommonRpcTransactionData {
                        hash: FixedBytes::<32>::ZERO,
                        nonce: 0, // TODO: Is this default value correct?
                        tx_type: 0,
                        gas_price: 0, // TODO: Is this default value correct?
                        gas_limit: 0,
                        max_fee_per_gas: 0, // TODO: Is this default value correct?
                        max_priority_fee_per_gas: 0, // TODO: Is this default value correct?
                        value: None,
                        access_list: AccessList::default(),
                        input: None,
                        r: Uint::<256, 4>::ZERO,
                        s: Uint::<256, 4>::ZERO,
                        v: false,
                        blob_versioned_hashes: Vec::new(),
                        authorization_list: Vec::new(),
                        block_hash,
                        block_number,
                        transaction_index,
                        effective_gas_price,
                        from,
                        to: TransactionTo::Address(Address::ZERO),
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
                                            hash: *signed.hash(),
                                            nonce: tx.nonce,
                                            tx_type: LEGACY_TX_TYPE_ID,
                                            gas_price: tx.gas_price,
                                            gas_limit: tx.gas_limit,
                                            input: Some(tx.input.clone()), // TODO: Remove clone
                                            value: Some(tx.value),
                                            r: signature.r(),
                                            s: signature.s(),
                                            v: signature.v(),
                                            to: TransactionTo::TxKind(tx.to),
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
                                            hash: *signed.hash(),
                                            nonce: tx.nonce,
                                            tx_type: EIP2930_TX_TYPE_ID,
                                            gas_price: tx.gas_price,
                                            gas_limit: tx.gas_limit,
                                            to: TransactionTo::TxKind(tx.to),
                                            value: Some(tx.value),
                                            access_list: tx.access_list.clone(), // TODO: Remove clone
                                            input: Some(tx.input.clone()),             // TODO: Remove clone
                                            r: signature.r(),
                                            s: signature.s(),
                                            v: signature.v(),
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
                                            nonce: tx.nonce,
                                            // tx_type: tx.tx_type(), // TODO: Not publicly accessible. Fix
                                            tx_type: EIP1559_TX_TYPE_ID,
                                            gas_limit: tx.gas_limit,
                                            max_fee_per_gas: tx.max_fee_per_gas,
                                            max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                                            to: TransactionTo::TxKind(tx.to),
                                            value: Some(tx.value),
                                            access_list: tx.access_list.clone(), // TODO: Remove clone
                                            input: Some(tx.input.clone()),             // TODO: Remove clone
                                            r: signature.r(),
                                            s: signature.s(),
                                            v: signature.v(),
                                            hash: *signed.hash(),
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
                                                nonce: tx.nonce,
                                                tx_type: EIP4844_TX_TYPE_ID,
                                                gas_limit: tx.gas_limit,
                                                max_fee_per_gas: tx.max_fee_per_gas,
                                                max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                                                to: TransactionTo::Address(tx.to),
                                                value: Some(tx.value),
                                                access_list: tx.access_list.clone(), // TODO: Remove clone
                                                blob_versioned_hashes: tx.blob_versioned_hashes.clone(), // TODO: Remove clone
                                                input: Some(tx.input.clone()),             // TODO: Remove clone
                                                r: signature.r(),
                                                s: signature.s(),
                                                v: signature.v(),
                                                hash: *signed.hash(),
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
                                                    nonce: tx.nonce,
                                                    tx_type: EIP4844_TX_TYPE_ID,
                                                    gas_limit: tx.gas_limit,
                                                    max_fee_per_gas: tx.max_fee_per_gas,
                                                    max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                                                    to: TransactionTo::Address(tx.to),
                                                    value: Some(tx.value),
                                                    access_list: tx.access_list.clone(), // TODO: Remove clone
                                                    blob_versioned_hashes: tx.blob_versioned_hashes.clone(), // TODO: Remove clone
                                                    input: Some(tx.input.clone()), // TODO: Remove clone
                                                    r: signature.r(),
                                                    s: signature.s(),
                                                    v: signature.v(),
                                                    hash: *signed.hash(),
                                                    ..common
                                                },
                                                max_fee_per_blob_gas: Some(tx.max_fee_per_blob_gas),
                                                blobs: tx_with_sidecar.sidecar.blobs.clone(), // TODO: Remove clone
                                                commitments: tx_with_sidecar.sidecar.commitments.clone(), // TODO: Remove clone
                                                proofs: tx_with_sidecar.sidecar.proofs.clone(), // TODO: Remove clone
                                            })
                                        }
                                    }
                                }
                                TxEnvelope::Eip7702(signed) => {
                                    let tx = signed.tx();
                                    let signature = signed.signature();

                                    RpcTransactionData::Ethereum(EthereumRpcTransactionData {
                                        common: CommonRpcTransactionData {
                                            nonce: tx.nonce,
                                            tx_type: EIP7702_TX_TYPE_ID,
                                            gas_limit: tx.gas_limit,
                                            max_fee_per_gas: tx.max_fee_per_gas,
                                            max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                                            to: TransactionTo::Address(tx.to),
                                            value: Some(tx.value),
                                            access_list: tx.access_list.clone(), // TODO: Remove clone
                                            authorization_list: tx.authorization_list.clone(), // TODO: Remove clone
                                            input: Some(tx.input.clone()),             // TODO: Remove clone
                                            r: signature.r(),
                                            s: signature.s(),
                                            v: signature.v(),
                                            hash: *signed.hash(),
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
                                    _ => unreachable!("Expected Ethereum transaction format for legacy transaction")
                                }
                            }
                        }
                        // Ethereum should never enter this match arm
                        // Other chains will enter this match arm for tx_type != Legacy
                        AnyTxEnvelope::Unknown(unknown) => {

                            let other_fields = &unknown.inner.fields;
                            let memo = &unknown.inner.memo;
                            let inner = &unknown.inner;
                            let ty = inner.ty;

                            let common_fields = CommonRpcTransactionData {
                                hash: unknown.hash,
                                nonce: other_fields
                                    .get_deserialized::<u64>("nonce")
                                    .and_then(|result| result.ok())
                                    .unwrap_or(0),
                                tx_type: ty.0, // Gets the first element of the tuple as u8
                                gas_price: other_fields
                                    .get_deserialized::<u128>("gasPrice")
                                    .and_then(|result| result.ok())
                                    .unwrap_or(0),
                                gas_limit: other_fields
                                    .get_deserialized::<u64>("gas")
                                    .and_then(|result| result.ok())
                                    .unwrap_or(0),
                                max_fee_per_gas: other_fields
                                    .get_deserialized::<u128>("maxFeePerGas")
                                    .and_then(|result| result.ok())
                                    .unwrap_or(0),
                                max_priority_fee_per_gas: other_fields
                                    .get_deserialized::<u128>("maxPriorityFeePerGas")
                                    .and_then(|result| result.ok())
                                    .unwrap_or(0),
                                value: other_fields
                                    .get_deserialized::<Uint<256, 4>>("value")
                                    .and_then(|result| result.ok()),
                                access_list: memo.access_list
                                    .get()
                                    .cloned()
                                    .unwrap_or(AccessList::default()),
                                input: other_fields
                                    .get_deserialized::<Bytes>("input")
                                    .and_then(|result| result.ok()),
                                r: Uint::<256, 4>::ZERO, // TODO: Fill this in
                                s: Uint::<256, 4>::ZERO, // TODO: Fill this in
                                v: false, // TODO: Fill this in
                                blob_versioned_hashes: memo.blob_versioned_hashes
                                    .get()
                                    .cloned()
                                    .unwrap_or(Vec::new()),
                                authorization_list: memo.authorization_list
                                    .get()
                                    .cloned()
                                    .unwrap_or(Vec::new()),
                                block_hash,
                                block_number,
                                transaction_index,
                                effective_gas_price,
                                from,
                                to: other_fields
                                    .get_deserialized::<TransactionTo>("to")
                                    .and_then(|result| result.ok())
                                    .unwrap_or(TransactionTo::Address(Address::ZERO)),
                            };

                            match chain {
                                Chain::Ethereum => {
                                    unreachable!("Ethereum transactions should be handled by AnyTxEnvelope::Ethereum variant")
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
                Err(anyhow!(
                    "Block contains only transaction hashes, full transaction data required"
                )) // Throw error if full tx objects are not included
            }
            BlockTransactions::Uncle => {
                Err(anyhow!("Uncle blocks not supported")) // TODO: Handle better
            }
        }
    }
}
