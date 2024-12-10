// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_network::{Network, primitives::BlockTransactionsKind};
use alloy_provider::{ext::DebugApi, Provider, ReqwestProvider};
use alloy_transport::{RpcError, Transport};
use alloy_rpc_types_eth::{Block, TransactionReceipt, Withdrawal};
use alloy_rpc_types_trace::{common::TraceResult, geth::{GethDebugTracingOptions, GethTrace}};

use eyre::Result;

use crate::parsers::blocks::BlockParser;
use crate::parsers::receipts::ReceiptParser;
use crate::types::blocks::{HeaderData, TransactionData, WithdrawalData};
use crate::types::receipts::{LogReceiptData, TransactionReceiptData};

#[derive(Debug)]
pub struct ParsedData {
    pub header: HeaderData,
    pub transactions: Vec<TransactionData>,
    pub withdrawals: Vec<WithdrawalData>,
    pub transaction_receipts: Vec<TransactionReceiptData>,
    pub logs: Vec<LogReceiptData>
}


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

pub async fn parse_data(block: Block, receipts: Vec<TransactionReceipt>) -> Result<ParsedData> {

    let header = block.clone().parse_header()?; //TODO: Remove clone
    let transactions = block.clone().parse_transactions()?;
    let withdrawals = block.clone().parse_withdrawals()?;

    let transaction_receipts = receipts.clone().parse_transaction_receipts()?;
    let logs = receipts.clone().parse_log_receipts()?;
    
    Ok(ParsedData { 
        header: header,
        transactions: transactions,
        withdrawals: withdrawals,
        transaction_receipts: transaction_receipts,
        logs: logs
    })
}