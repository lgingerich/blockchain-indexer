// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use alloy_consensus::{TxEip4844Variant, TxEnvelope};
use alloy_eips::eip2930::AccessList;
use alloy_eips::eip7702::SignedAuthorization;
use alloy_network::primitives::BlockTransactions;
use alloy_network::{AnyRpcBlock, AnyRpcHeader, AnyRpcTransaction, AnyTxEnvelope};
use alloy_primitives::{Address, Bytes, FixedBytes, Uint};
use alloy_rpc_types_eth::{Block, Header, Withdrawals};
use alloy_serde::WithOtherFields;
use anyhow::{anyhow, Result};
use chrono::DateTime;
use tracing::info;

use crate::utils::hex_to_u64;
use crate::models::common::TransactionTo;

use crate::models::datasets::blocks::{RpcHeaderData, RpcWithdrawalData};
use crate::models::datasets::transactions::RpcTransactionData;

// NOTE: No handling for uncle blocks
pub trait BlockParser {
    fn parse_header(self) -> Result<Vec<RpcHeaderData>>;
    fn parse_transactions(self) -> Result<Vec<RpcTransactionData>>;
    fn parse_withdrawals(self) -> Result<Vec<RpcWithdrawalData>>;
}

impl BlockParser for AnyRpcBlock {
    fn parse_header(self) -> Result<Vec<RpcHeaderData>> {
        let inner = self.header.inner.clone();
        let other = self.other.clone();

        // TODO: Add error handling
        Ok(vec![RpcHeaderData {
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
            target_blobs_per_block: inner.target_blobs_per_block,
            total_difficulty: self.header.total_difficulty,
            size: self.header.size,

            // ZKsync fields
            l1_batch_number: other
                .get_deserialized::<String>("l1BatchNumber")
                .and_then(|result| result.ok())
                .and_then(|hex| hex_to_u64(hex)),
            l1_batch_timestamp: other
                .get_deserialized::<String>("l1BatchTimestamp")
                .and_then(|result| result.ok())
                .and_then(|hex| hex_to_u64(hex))
                .and_then(|timestamp| DateTime::from_timestamp(timestamp as i64, 0)),
            seal_fields: other.get_deserialized::<Vec<String>>("sealFields").and_then(|result| result.ok()),

        }])
    }

    fn parse_transactions(self) -> Result<Vec<RpcTransactionData>> {
        match self.transactions {
            BlockTransactions::Full(_) => Ok(self
                .transactions   
                .txns() 
                .map(|transaction| {
                    let fields: RpcTransactionData = RpcTransactionData {
                        nonce: 0,
                        gas_price: 0,
                        tx_type: 0,
                        gas_limit: 0,
                        max_fee_per_gas: 0,
                        max_priority_fee_per_gas: 0,
                        to: TransactionTo::Address(Address::ZERO),
                        value: Uint::<256, 4>::ZERO,
                        access_list: AccessList::default(),
                        authorization_list: Vec::new(),
                        blob_versioned_hashes: Vec::new(),
                        max_fee_per_blob_gas: 0,
                        blobs: Vec::new(),
                        commitments: Vec::new(),
                        proofs: Vec::new(),
                        input: Bytes::default(),
                        r: Uint::<256, 4>::ZERO,
                        s: Uint::<256, 4>::ZERO,
                        v: false, // TODO: False likely isn't the correct default value
                        hash: FixedBytes::<32>::ZERO,
                        block_hash: Some(FixedBytes::<32>::ZERO),
                        block_number: None,
                        transaction_index: None,
                        effective_gas_price: None,
                        from: Address::ZERO,
                        l1_batch_number: Some(0),
                        l1_batch_tx_index: Some(0),
                    };

/*
                    AnyTxEnvelope
                        Ethereum(TxEnvelope)
                            Legacy(Signed<TxLegacy>),
                            Eip2930(Signed<TxEip2930>),
                            Eip1559(Signed<TxEip1559>),
                            Eip4844(Signed<TxEip4844Variant>),
                            Eip7702(Signed<TxEip7702>),
                        Unknown(UnknownTxEnvelope)
                            pub hash: FixedBytes<32>,
                            pub inner: UnknownTypedTransaction,
                                    pub ty: AnyTxType,
                                    pub fields: OtherFields,
                                    pub memo: DeserMemo,
                        
*/
                    match &transaction.inner.inner {
                        AnyTxEnvelope::Ethereum(inner) => {
                            match inner {
                                TxEnvelope::Legacy(signed) => {
                                    let tx = signed.tx();
                                    let signature = signed.signature();

                                    RpcTransactionData {
                                        nonce: tx.nonce,
                                        gas_price: tx.gas_price,
                                        gas_limit: tx.gas_limit,
                                        to: TransactionTo::TxKind(tx.to),
                                        value: tx.value,
                                        input: tx.input.clone(), // TODO: Remove clone
                                        r: signature.r(),
                                        s: signature.s(),
                                        v: signature.v(),
                                        hash: *signed.hash(),
                                        ..fields
                                    }
                                }
                                TxEnvelope::Eip2930(signed) => {
                                    let tx = signed.tx();
                                    let signature = signed.signature();

                                    RpcTransactionData {
                                        nonce: tx.nonce,
                                        gas_price: tx.gas_price,
                                        gas_limit: tx.gas_limit,
                                        to: TransactionTo::TxKind(tx.to),
                                        value: tx.value,
                                        access_list: tx.access_list.clone(), // TODO: Remove clone
                                        input: tx.input.clone(),             // TODO: Remove clone
                                        r: signature.r(),
                                        s: signature.s(),
                                        v: signature.v(),
                                        hash: *signed.hash(),
                                        ..fields
                                    }
                                }
                                TxEnvelope::Eip1559(signed) => {
                                    let tx = signed.tx();
                                    let signature = signed.signature();

                                    RpcTransactionData {
                                        nonce: tx.nonce,
                                        gas_limit: tx.gas_limit,
                                        max_fee_per_gas: tx.max_fee_per_gas,
                                        max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                                        to: TransactionTo::TxKind(tx.to),
                                        value: tx.value,
                                        access_list: tx.access_list.clone(), // TODO: Remove clone
                                        input: tx.input.clone(),             // TODO: Remove clone
                                        r: signature.r(),
                                        s: signature.s(),
                                        v: signature.v(),
                                        hash: *signed.hash(),
                                        ..fields
                                    }
                                }
                                TxEnvelope::Eip4844(signed) => {
                                    let signature = signed.signature();

                                    match signed.tx() {
                                        TxEip4844Variant::TxEip4844(tx) => RpcTransactionData {
                                            nonce: tx.nonce,
                                            gas_limit: tx.gas_limit,
                                            max_fee_per_gas: tx.max_fee_per_gas,
                                            max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                                            to: TransactionTo::Address(tx.to),
                                            value: tx.value,
                                            access_list: tx.access_list.clone(), // TODO: Remove clone
                                            blob_versioned_hashes: tx.blob_versioned_hashes.clone(), // TODO: Remove clone
                                            max_fee_per_blob_gas: tx.max_fee_per_blob_gas,
                                            input: tx.input.clone(), // TODO: Remove clone
                                            r: signature.r(),
                                            s: signature.s(),
                                            v: signature.v(),
                                            hash: *signed.hash(),
                                            ..fields
                                        },
                                        TxEip4844Variant::TxEip4844WithSidecar(tx_with_sidecar) => {
                                            let tx = &tx_with_sidecar.tx;
                                            RpcTransactionData {
                                                nonce: tx.nonce,
                                                gas_limit: tx.gas_limit,
                                                max_fee_per_gas: tx.max_fee_per_gas,
                                                max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                                                value: tx.value,
                                                access_list: tx.access_list.clone(), // TODO: Remove clone
                                                blob_versioned_hashes: tx
                                                    .blob_versioned_hashes
                                                    .clone()
                                                    .clone(), // TODO: Remove clone
                                                max_fee_per_blob_gas: tx.max_fee_per_blob_gas,
                                                blobs: tx_with_sidecar.sidecar.blobs.clone(), // TODO: Remove clone
                                                commitments: tx_with_sidecar.sidecar.commitments.clone(), // TODO: Remove clone
                                                proofs: tx_with_sidecar.sidecar.proofs.clone(), // TODO: Remove clone
                                                input: tx.input.clone(), // TODO: Remove clone
                                                r: signature.r(),
                                                s: signature.s(),
                                                v: signature.v(),
                                                hash: *signed.hash(),
                                                ..fields
                                            }
                                        }
                                    }
                                }
                                TxEnvelope::Eip7702(signed) => {
                                    let tx = signed.tx();
                                    let signature = signed.signature();

                                    RpcTransactionData {
                                        nonce: tx.nonce,
                                        gas_limit: tx.gas_limit,
                                        max_fee_per_gas: tx.max_fee_per_gas,
                                        max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                                        to: TransactionTo::Address(tx.to),
                                        value: tx.value,
                                        access_list: tx.access_list.clone(), // TODO: Remove clone
                                        authorization_list: tx.authorization_list.clone(), // TODO: Remove clone
                                        input: tx.input.clone(), // TODO: Remove clone
                                        r: signature.r(),
                                        s: signature.s(),
                                        v: signature.v(),
                                        hash: *signed.hash(),
                                        ..fields
                                    }
                                }
                                // TODO: Use better default values for undefined TxEnvelope types
                                // Can I do an if...else? Try to access common fields that are likely to exist in all transaction types, and if they don't exist, use empty/zero defaults
                                // Maybe I only give defaults for fields guaranteed to exist in all transaction types?
                                _ => RpcTransactionData {
                                    nonce: 0,
                                    gas_limit: 0,
                                    value: Uint::<256, 4>::ZERO,
                                    input: Bytes::new(),
                                    r: Uint::<256, 4>::ZERO,
                                    s: Uint::<256, 4>::ZERO,
                                    v: false,
                                    hash: FixedBytes::<32>::ZERO,
                                    ..fields
                                },
                            }
                        }
                        AnyTxEnvelope::Unknown(unknown) => {
                            info!("Unknown transaction envelope: {:?}", unknown);

                            let other_fields = &unknown.inner.fields;
                            let memo = &unknown.inner.memo;
                            let inner = &unknown.inner;
                            let ty = inner.ty;
                            
                            RpcTransactionData {
                                hash: unknown.hash,
                                tx_type: ty.0, // Gets the first element of the tuple as u8
                                // gas: fields
                                //     .get_deserialized::<>("gas")
                                //     .and_then(|result| result.ok())
                                //     .unwrap_or(),
                                gas_price: other_fields
                                    .get_deserialized::<u128>("gasPrice")
                                    .and_then(|result| result.ok())
                                    .unwrap_or(0),
                                input: other_fields
                                    .get_deserialized::<Bytes>("input")
                                    .and_then(|result| result.ok())
                                    .unwrap_or(Bytes::default()),
                                // input: fields // Try fields first, then fall back to memo if not found
                                //     .get_deserialized::<Bytes>("input")
                                //     .and_then(|result| result.ok())
                                //     .or_else(|| memo.input.get().cloned())
                                //     .unwrap_or(Bytes::default()),                                      
                                l1_batch_number: other_fields
                                    .get_deserialized::<String>("l1BatchNumber")
                                    .and_then(|result| result.ok())
                                    .and_then(|hex| hex_to_u64(hex)),
                                    // .unwrap_or(0),
                                l1_batch_tx_index: other_fields
                                    .get_deserialized::<String>("l1BatchTxIndex")
                                    .and_then(|result| result.ok())
                                    .and_then(|hex| hex_to_u64(hex)),
                                    // .unwrap_or(TransactionTo::Address(Address::ZERO)),                                                                    
                                max_fee_per_gas: other_fields
                                    .get_deserialized::<u128>("maxFeePerGas")
                                    .and_then(|result| result.ok())
                                    .unwrap_or(0), // not sure if this field should be option or not                                 
                                max_priority_fee_per_gas: other_fields
                                    .get_deserialized::<u128>("maxPriorityFeePerGas")
                                    .and_then(|result| result.ok())
                                    .unwrap_or(0),                                    
                                nonce: other_fields
                                    .get_deserialized::<u64>("nonce")
                                    .and_then(|result| result.ok())
                                    .unwrap_or(0),                                
                                to: other_fields
                                    .get_deserialized::<TransactionTo>("to")
                                    .and_then(|result| result.ok())
                                    .unwrap_or(TransactionTo::Address(Address::ZERO)),
                                value: other_fields
                                    .get_deserialized::<Uint<256, 4>>("value")
                                    .and_then(|result| result.ok())
                                    .unwrap_or(Uint::<256, 4>::ZERO),

                                access_list: memo.access_list
                                    .get()
                                    .cloned()
                                    .unwrap_or_default(),
                                blob_versioned_hashes: memo.blob_versioned_hashes
                                    .get()
                                    .cloned()
                                    .unwrap_or_default(),                                    
                                authorization_list: memo.authorization_list
                                    .get()
                                    .cloned()
                                    .unwrap_or_default(),
                                ..fields
                            }
                        }
                    }
                })
                .collect()),
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

    fn parse_withdrawals(self) -> Result<Vec<RpcWithdrawalData>> {
        Ok(self
            .withdrawals
            .clone()
            .map(|withdrawals| {
                withdrawals
                    .0
                    .into_iter()
                    .map(|withdrawal| RpcWithdrawalData {
                        index: withdrawal.index,
                        validator_index: withdrawal.validator_index,
                        address: withdrawal.address,
                        amount: withdrawal.amount,
                    })
                    .collect::<Vec<RpcWithdrawalData>>()
            })
            .unwrap_or_default())
    }
}
