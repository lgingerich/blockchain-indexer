#[derive(Debug)]
pub struct BaseBlock {
    base_fee_per_gas: Option<u64>,
    block_hash: BlockHash,
    block_number: BlockNumber,
    block_date: 
    block_time: BlockTimestamp,
    difficulty: U256,
    extra_data: Option<>,
    gas_limit: u64,
    gas_used: u64,
    logs_bloom: String,
    miner: 
    mix_hash: 
    nonce: 
    parent_hash: 
    receipts_root: 
    sha3_uncles: 
    size: u64,
    state_root: 
    total_difficulty: U256,
    transactions: Vec<>,
    transactions_root: 
    uncles: 
}

#[derive(Debug)]
pub struct ArbitrumBlock {
    base: BaseBlock,
    l1_block_number: u64,
    send_count: Option<u64>,
    send_root: 
}

#[derive(Debug)]
pub struct EthereumBlock {
    base: BaseBlock,
    blob_gas_used: Option<u64>,
    excess_blob_gas: Option<u64>,
    parent_beacon_block_root: 
    withdrawals: 
    withdrawals_root: 
}

#[derive(Debug)]
pub struct ZkSyncBlock {
    base: BaseBlock,
    l1_batch_number: Option<u64>,
    l1_batch_time: Option<BlockTimestamp>,
    seal_fields: 
}