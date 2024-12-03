mod indexer;

use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    network::primitives::BlockTransactionsKind,
    providers::ProviderBuilder,
};
use eyre::Result;

// Alloy
    // Crate: `primitives`
        // Good for data types
    // Crate: `providers`
        // Has all the standard rpc methods



// RPC URL
const RPC_URL: &str = "https://eth.llamarpc.com";

#[tokio::main]
async fn main() -> Result<()> {
    // Create a RPC provider using HTTP with the `reqwest` crate
    let rpc_url = RPC_URL.parse()?;
    let provider = ProviderBuilder::new().on_http(rpc_url);

    // Get latest block number
    let latest_block: BlockNumberOrTag = indexer::get_latest_block_number(&provider).await?;
    // let latest_block: BlockNumberOrTag = BlockNumberOrTag::Number(1000000);

    // Get block by number
    let kind = BlockTransactionsKind::Full; // Hashes: only include tx hashes, Full: include full tx objects
    let block = indexer::get_block_by_number(&provider, latest_block, kind).await?;

    // Get receipts by block number
    let block_id = BlockId::Number(latest_block);
    let receipts = indexer::get_block_receipts(&provider, block_id).await?;
    
    println!("Latest block number: {:?}", latest_block);
    println!("Block: {:?}", block);
    println!("Receipts: {:?}", receipts);
    Ok(())
}