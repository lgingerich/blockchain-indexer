// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use alloy_primitives::FixedBytes;
use alloy_network::primitives::BlockTransactions;
use alloy_rpc_types_eth::{Block, Transaction, Withdrawal, Withdrawals, Header};
use eyre::Result;
use chrono::DateTime;
use alloy_consensus::TxEnvelope;
use alloy_primitives::{Uint, Bytes};

use crate::types::blocks::{HeaderData, TransactionData};


pub trait BlockParser {
    fn parse_header(self) -> Result<HeaderData>;
    fn parse_transactions(self) -> Result<Vec<TransactionData>>;
    // fn parse_uncles(self) -> Result<Vec<String>>;
    // fn parse_withdrawals(self) -> Result<Option<Vec<Withdrawal>>>;
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
            // timestamp: inner.timestamp,
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

    // fn parse_transactions(self) -> Result<Vec<TransactionData>> {
    //     println!("transactions: {:?}", self.transactions);
    //     Ok(vec![])
    // }


    fn parse_transactions(self) -> Result<Vec<TransactionData>> {
        match self.transactions {
            BlockTransactions::Full(_) => Ok(self.transactions.txns().map(|transaction| {
                let inner: TxEnvelope = transaction.inner.clone(); // TODO: Remove clone
                // let inner: TxEnvelope = transaction.inner;

                match &inner {
                    TxEnvelope::Legacy(signed) => {
                        let tx = signed.tx();
                        TransactionData {
                            nonce: tx.nonce,
                            gas_limit: tx.gas_limit,
                            value: tx.value,
                            input: tx.input.clone(),
                        }
                    },
                    _ => {
                        TransactionData {
                            nonce: 0,
                            gas_limit: 0,
                            value: Uint::<256, 4>::ZERO,
                            input: Bytes::new(),
                        }
                    }, // TODO: Use better default values for undefined TxEnvelope types
                }
                
                // // Extract common fields based on envelope type
                // let (chain_id, nonce, gas_limit, value, input) = match &inner {
                //     TxEnvelope::Legacy(signed) => {
                //         let tx = signed.tx();
                //         (tx.chain_id, tx.nonce, tx.gas_limit, 
                //         tx.value, tx.input.clone())
                //     },
                //     TxEnvelope::Eip2930(signed) => {
                //         let tx = signed.tx();
                //         (Some(tx.chain_id), tx.nonce, tx.gas_limit, 
                //         tx.value, tx.input.clone())
                //     },
                //     TxEnvelope::Eip1559(signed) => {
                //         let tx = signed.tx();
                //         (Some(tx.chain_id), tx.nonce, tx.gas_limit, 
                //         tx.value, tx.input.clone())
                //     },
                //     // TxEnvelope::Eip4844(signed) => {
                //     //     let tx = signed.tx();
                //     //     (Some(tx.chain_id), tx.nonce, tx.gas_limit, 
                //     //     tx.value, tx.input.clone())
                //     // },
                //     TxEnvelope::Eip7702(signed) => {
                //         let tx = signed.tx();
                //         (Some(tx.chain_id), tx.nonce, tx.gas_limit, 
                //         tx.value, tx.input.clone())
                //     },
                //     _ => {
                //         (None, 0, 0, Uint::<256, 4>::ZERO, Bytes::new())
                //     }, // TODO: Use better default values for undefined TxEnvelope types
                // }
            }).collect()),
            BlockTransactions::Hashes(_) => {
                Err(eyre::eyre!("Block contains only transaction hashes, full transaction data required")) // Throw error if full tx objects are not included
            },
            BlockTransactions::Uncle => {
                Err(eyre::eyre!("Uncle blocks not supported")) // TODO: Handle better
            }
        }
    }


    // fn parse_withdrawals(self) -> Result<Option<Vec<Withdrawal>>> {
    //     Ok(self.withdrawals.map(|w| w.into_iter().collect()))
    // }

}