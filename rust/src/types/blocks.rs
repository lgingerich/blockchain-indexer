// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use alloy_eips::eip2930::AccessList;
use alloy_eips::eip7702::SignedAuthorization;
use alloy_primitives::{Address, Bloom, Bytes, FixedBytes, TxKind, Uint};

use chrono::{DateTime, NaiveDate,Utc};

#[derive(Debug)]
pub enum ChainId {
    Legacy(Option<u64>),  // For TxLegacy where chain_id is Option<u64>
    Other(u64)           // For all other tx types where chain_id is u64
}

#[derive(Debug)]
pub struct HeaderData {
    pub hash: FixedBytes<32>,
    pub parent_hash: FixedBytes<32>,
    pub ommers_hash: FixedBytes<32>,
    pub beneficiary: Address,
    pub state_root: FixedBytes<32>,
    pub transactions_root: FixedBytes<32>,
    pub receipts_root: FixedBytes<32>,
    pub logs_bloom: Bloom,
    pub difficulty: Uint<256, 4>,
    pub number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: DateTime<Utc>,
    pub date: NaiveDate,
    pub extra_data: Bytes,
    pub mix_hash: FixedBytes<32>,
    pub nonce: FixedBytes<8>,
    pub base_fee_per_gas: Option<u64>,
    pub withdrawals_root: Option<FixedBytes<32>>,
    pub blob_gas_used: Option<u64>,
    pub excess_blob_gas: Option<u64>,
    pub parent_beacon_block_root: Option<FixedBytes<32>>,
    pub requests_hash: Option<FixedBytes<32>>,
    pub target_blobs_per_block: Option<u64>,
    pub total_difficulty: Option<Uint<256, 4>>,
    pub size: Option<Uint<256, 4>>,
}

#[derive(Debug)]
pub struct TransactionData {
    pub chain_id: ChainId,
    pub nonce: u64,
    pub gas_limit: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    // pub to: TxKind,
    pub value: Uint<256, 4>,
    pub access_list: AccessList,
    pub authorization_list: Vec<SignedAuthorization>,
    pub input: Bytes,
    pub r: Uint<256, 4>,
    pub s: Uint<256, 4>,
    // pub v: bool,
    pub hash: FixedBytes<32>,

    pub block_hash: Option<FixedBytes<32>>,
    pub block_number: Option<u64>,
    pub transaction_index: Option<u64>,
    pub effective_gas_price: Option<u128>,
    pub from: Address,
}
