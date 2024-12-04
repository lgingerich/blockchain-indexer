// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]

use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_network::{Network, primitives::BlockTransactionsKind};
use alloy_provider::{ext::DebugApi, Provider, ReqwestProvider};
use alloy_transport::{RpcError, Transport};
use alloy_rpc_types_eth::{Block, TransactionReceipt, Header, Withdrawals};
use alloy_rpc_types_trace::{common::TraceResult, geth::{GethDebugTracingOptions, GethTrace}};

use eyre::Result;
use std::collections::HashMap;


#[derive(Debug)]
pub struct ParsedBlock {
    header: Header,
    withdrawals: Option<Withdrawals>,
}


// // https://docs.rs/alloy/latest/alloy/rpc/types/struct.Block.html#
// pub struct Block<T = Transaction, H = Header> {
//     pub header: H,
//     pub uncles: Vec<FixedBytes<32>>,
//     pub transactions: BlockTransactions<T>,
//     pub withdrawals: Option<Withdrawals>,
// }



/// Retrieves the latest block number from the blockchain
///
/// # Arguments
/// * `provider` - The blockchain provider implementation
///
/// # Returns
/// * `Result<BlockNumberOrTag>` - The latest block number wrapped in BlockNumberOrTag enum
pub async fn get_latest_block_number(provider: &ReqwestProvider) -> Result<BlockNumberOrTag> { // TODO: Why do I use ReqwestProvider here?
    let latest_block = provider.get_block_number().await?;
    Ok(BlockNumberOrTag::Number(latest_block)) // TODO: Why do I wrap this but not other results?
}

/// Fetches a block by its block number
///
/// # Arguments
/// * `provider` - The blockchain provider implementation
/// * `block_number` - The block number to fetch, can be a specific number or tag (latest, earliest, etc.)
/// * `kind` - Specifies whether to include full transaction objects or just transaction hashes
///
/// # Returns
/// * `Result<Option<N::BlockResponse>>` - The block data if found, None if the block doesn't exist
pub async fn get_block_by_number<T, N>(
    provider: &dyn Provider<T, N>, 
    block_number: BlockNumberOrTag,
    kind: BlockTransactionsKind,
) -> Result<Option<N::BlockResponse>> 
where
    T: Transport + Clone,
    N: Network,
{
    let block = provider.get_block_by_number(block_number, kind).await?;
    Ok(block)
}

/// Retrieves all transaction receipts for a given block
///
/// # Arguments
/// * `provider` - The blockchain provider implementation
/// * `block` - The block identifier (can be hash or number)
///
/// # Returns
/// * `Result<Option<Vec<N::ReceiptResponse>>>` - Vector of transaction receipts if the block exists
pub async fn get_block_receipts<T, N>(
    provider: &dyn Provider<T, N>,
    block: BlockId,
) -> Result<Option<Vec<N::ReceiptResponse>>>
where
    T: Transport + Clone,
    N: Network,
{
    let receipts = provider.get_block_receipts(block).await?;
    Ok(receipts)
}




// pub async fn parse_block(block: Block) -> Result<ParsedBlock> {
pub async fn parse_block(block: Block) -> Result<String> {
    let block_header = block.header;
    let uncles = block.uncles;
    let transactions = block.transactions;
    let withdrawals = block.withdrawals;
    let total_difficulty = block.header.total_difficulty;
    let size = block.header.size;

    // // This just repacks the block data — not useful
    // Ok(ParsedBlock {
    //     header: block_data,
    //     withdrawals: withdrawals,
    // })

    Ok(total_difficulty.expect("Total difficulty is None").to_string())
}