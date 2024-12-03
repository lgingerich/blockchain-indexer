// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]

mod indexer;

use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_network::primitives::BlockTransactionsKind;
use alloy_provider::ProviderBuilder;
use alloy_rpc_types_trace::geth::GethDebugTracingOptions;

use eyre::Result;

// RPC URL
const RPC_URL: &str = "https://eth.drpc.org";

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