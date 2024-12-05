// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use alloy_consensus::TxEnvelope;
use alloy_eips::eip2930::AccessList;
use alloy_eips::eip7702::SignedAuthorization;
use alloy_network::primitives::BlockTransactions;
use alloy_primitives::{Bytes, FixedBytes, Uint};
use alloy_provider::utils::Eip1559Estimation;
use alloy_rpc_types_eth::{Block, Header};

use eyre::Result;
use chrono::DateTime;

use crate::types::blocks::{ChainId, HeaderData, TransactionData};

pub trait BlockParser {
    fn parse_header(self) -> Result<HeaderData>;
    fn parse_transactions(self) -> Result<Vec<TransactionData>>;
}

impl BlockParser for Block {
    
    fn parse_header(self) -> Result<HeaderData> {
        let inner = self.header.inner;

        Ok(HeaderData {
            hash: self.header.hash,
            parent_hash: inner.parent_hash,
            ommers_hash: inner.ommers_hash,
            beneficiary: inner.beneficiary,
            state_root: inner.state_root,
            transactions_root: inner.transactions_root,
            receipts_root: inner.receipts_root,
            logs_bloom: inner.logs_bloom,
            difficulty: inner.difficulty,
            number: inner.number,
            gas_limit: inner.gas_limit,
            gas_used: inner.gas_used,
            timestamp: DateTime::from_timestamp(inner.timestamp as i64, 0).expect("invalid timestamp"),
            date: DateTime::from_timestamp(inner.timestamp as i64, 0).expect("invalid timestamp").date_naive(),
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
        })
    }

    fn parse_transactions(self) -> Result<Vec<TransactionData>> {
        match self.transactions {
            BlockTransactions::Full(_) => Ok(self.transactions.txns().map(|transaction| {
                
                let fields = TransactionData {
                    chain_id: ChainId::Other(0),
                    nonce: 0,
                    gas_limit: 0,
                    max_fee_per_gas: 0,
                    max_priority_fee_per_gas: 0,
                    // to: TxKind::default(),
                    value: Uint::<256, 4>::ZERO,
                    access_list: AccessList::default(),
                    authorization_list: Vec::new(),
                    input: Bytes::default(),
                    r: Uint::<256, 4>::ZERO,
                    s: Uint::<256, 4>::ZERO,
                    // v: false,
                    hash: FixedBytes::<32>::ZERO,
                    block_hash: transaction.block_hash,
                    block_number: transaction.block_number,
                    transaction_index: transaction.transaction_index,
                    effective_gas_price: transaction.effective_gas_price,
                    from: transaction.from,
                };
                
                
                let inner: TxEnvelope = transaction.inner.clone(); // TODO: Remove clone

                match &inner {
                    TxEnvelope::Legacy(signed) => {
                        let tx = signed.tx();
                        let signature = signed.signature();

                        TransactionData {
                            chain_id: ChainId::Legacy(tx.chain_id),
                            nonce: tx.nonce,
                            gas_limit: tx.gas_limit,
                            value: tx.value,
                            input: tx.input.clone(), // TODO: Remove clone
                            r: signature.r(),
                            s: signature.s(),
                            // v: signature.v(),
                            // TODO: Add y_parity
                            hash: signed.hash().clone(), // TODO: Remove clone
                            ..fields
                        }
                    },
                    TxEnvelope::Eip2930(signed) => {
                        let tx = signed.tx();
                        let signature = signed.signature();

                        TransactionData {
                            chain_id: ChainId::Other(tx.chain_id),
                            nonce: tx.nonce,
                            gas_limit: tx.gas_limit,
                            value: tx.value,
                            access_list: tx.access_list.clone(), // TODO: Remove clone
                            input: tx.input.clone(), // TODO: Remove clone
                            r: signature.r(),
                            s: signature.s(),
                            // v: signature.v(),
                            // TODO: Add y_parity
                            hash: signed.hash().clone(), // TODO: Remove clone
                            ..fields
                        }
                    },
                    TxEnvelope::Eip1559(signed) => {
                        let tx = signed.tx();
                        let signature = signed.signature();

                        TransactionData {
                            chain_id: ChainId::Other(tx.chain_id),
                            nonce: tx.nonce,
                            gas_limit: tx.gas_limit,
                            max_fee_per_gas: tx.max_fee_per_gas,
                            max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                            // to: tx.to,
                            value: tx.value,
                            access_list: tx.access_list.clone(), // TODO: Remove clone
                            input: tx.input.clone(), // TODO: Remove clone
                            r: signature.r(),
                            s: signature.s(),
                            // v: signature.v(),
                            // TODO: Add y_parity
                            hash: signed.hash().clone(), // TODO: Remove clone
                            ..fields
                        }
                    },
                    // TODO: Implement EIP4844 and EIP4844Sidecar


                    TxEnvelope::Eip7702(signed) => {
                        let tx = signed.tx();
                        let signature = signed.signature();

                        TransactionData {
                            chain_id: ChainId::Other(tx.chain_id),
                            nonce: tx.nonce,
                            gas_limit: tx.gas_limit,
                            max_fee_per_gas: tx.max_fee_per_gas,
                            max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                            // to: tx.to,
                            value: tx.value,
                            access_list: tx.access_list.clone(), // TODO: Remove clone
                            authorization_list: tx.authorization_list.clone(), // TODO: Remove clone
                            input: tx.input.clone(), // TODO: Remove clone
                            r: signature.r(),
                            s: signature.s(),
                            // v: signature.v(),
                            // TODO: Add y_parity
                            hash: signed.hash().clone(), // TODO: Remove clone
                            ..fields
                        }
                    },
                    // TODO: Use better default values for undefined TxEnvelope types
                    // Can I do an if...else? Try to access common fields that are likely to exist in all transaction types, and if they don't exist, use empty/zero defaults
                    // Maybe I only give defaults for fields guaranteed to exist in all transaction types?
                    _ => {
                        TransactionData {
                            chain_id: ChainId::Other(0),
                            nonce: 0,
                            gas_limit: 0,
                            value: Uint::<256, 4>::ZERO,
                            input: Bytes::new(),
                            r: Uint::<256, 4>::ZERO,
                            s: Uint::<256, 4>::ZERO,
                            // v: false,
                            hash: FixedBytes::<32>::ZERO,
                            ..fields
                        }
                    },
                }
            }).collect()),
            BlockTransactions::Hashes(_) => {
                Err(eyre::eyre!("Block contains only transaction hashes, full transaction data required")) // Throw error if full tx objects are not included
            },
            BlockTransactions::Uncle => {
                Err(eyre::eyre!("Uncle blocks not supported")) // TODO: Handle better
            }
        }
    }

}