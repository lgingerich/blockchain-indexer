use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    network::{Network, primitives::BlockTransactionsKind},
    providers::Provider,
    transports::Transport
};
use eyre::Result;

/// Retrieves the latest block number from the blockchain
///
/// # Arguments
/// * `provider` - The blockchain provider implementation
///
/// # Returns
/// * `Result<BlockNumberOrTag>` - The latest block number wrapped in BlockNumberOrTag enum
pub async fn get_latest_block_number<T, N>(provider: &dyn Provider<T, N>) -> Result<BlockNumberOrTag> 
where
    T: Transport + Clone,
    N: Network,
{
    let latest_block = provider.get_block_number().await?;
    Ok(BlockNumberOrTag::Number(latest_block))
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