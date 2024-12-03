use chrono::{NaiveDate, NaiveDateTime};
use alloy::eips::eip2930::AccessListItem;

#[derive(Debug)]
pub struct BaseTransaction {
    // Fields from eth_getBlockByNumber
    block_hash: String,
    block_number: u64,
    block_time: NaiveDateTime,
    block_date: NaiveDate,
    chain_id: Option<u64>,
    from_address: String,
    gas_limit: u64,
    gas_price: u64,
    hash: String,
    input: String,
    nonce: u64,
    r: Option<String>,
    s: Option<String>,
    to_address: String,
    transaction_index: u64,
    v: Option<u64>,
    value: U256,

    // Fields from eth_getTransactionReceipt
    status: u64,
    cumulative_gas_used: u64,
    effective_gas_price: u64,
    gas_used: u64,
    logs_bloom: String, // check if this is always the same as BaseBlock.logs_bloom and delete if so
    contract_address: Option<String>, // is this needed? is this a duplicate?
}

#[derive(Debug)]
pub struct ArbitrumTransaction {
    base: BaseTransaction,

    // Fields from eth_getTransactionReceipt
    blob_gas_used: Option<u64>, // check if these 3 are actually optional
    l1_block_number: Option<u64>,
    gas_used_for_l1: Option<u64>,
}

#[derive(Debug)]
pub struct EthereumTransaction {
    base: BaseTransaction,

    // Fields from eth_getBlockByNumber
    access_list: Option<Vec<AccessListItem>>,
    blob_versioned_hashes: Option<Vec<String>>,
    max_fee_per_blob_gas: Option<u64>,
    max_fee_per_gas: Option<u64>,
    max_priority_fee_per_gas: Option<u64>,
    y_parity: Option<u64>,
}

#[derive(Debug)]
pub struct ZKsyncTransaction {
    // Fields from eth_getBlockByNumber
    base: BaseTransaction,

    // Fields from eth_getTransactionReceipt
}
