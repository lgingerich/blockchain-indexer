use chrono::{NaiveDate, NaiveDateTime};
use Transaction, Withdrawal; // TODO: Add these — idk if alloy has traits or I need to define them

#[derive(Debug)]
pub struct BaseBlock {
    base_fee_per_gas: Option<u64>,
    block_hash: BlockHash,
    block_number: BlockNumber,
    block_date: NaiveDate,
    block_time: NaiveDateTime,
    difficulty: U256,
    extra_data: Option<String>,
    gas_limit: u64,
    gas_used: u64,
    logs_bloom: String,
    miner: String,
    mix_hash: String,
    nonce: String,
    parent_hash: String,
    receipts_root: String,
    sha3_uncles: String,
    size: u64,
    state_root: String,
    total_difficulty: U256,
    transactions: Vec<Transaction>, // Transaction trait/type needs to handle full tx objects or only hashes
    transactions_root: String,
    uncles: Vec<String>,
}

#[derive(Debug)]
pub struct ArbitrumBlock {
    base: BaseBlock,
    l1_block_number: u64,
    send_count: Option<u64>,
    send_root: Option<String>,
}

#[derive(Debug)]
pub struct EthereumBlock {
    base: BaseBlock,
    blob_gas_used: Option<u64>,
    excess_blob_gas: Option<u64>,
    parent_beacon_block_root: Option<String>,
    withdrawals: Option<Vec<Withdrawal>>, // TODO: Add Withdrawal trait
    withdrawals_root: Option<String>,
}

#[derive(Debug)]
pub struct ZkSyncBlock {
    base: BaseBlock,
    l1_batch_number: Option<u64>,
    l1_batch_time: Option<NaiveDateTime>,
    seal_fields: Vec<String>,
}