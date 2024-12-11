// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

mod indexer;
mod models;

use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_network::primitives::BlockTransactionsKind;
use alloy_provider::ProviderBuilder;
use alloy_rpc_types_eth::{Block, TransactionReceipt};
use alloy_rpc_types_trace::geth::GethDebugTracingOptions;

use eyre::Result;


const RPC_URL: &str = "https://eth.drpc.org";


#[tokio::main]
async fn main() -> Result<()> {
    // Create a RPC provider using HTTP with the `reqwest` crate
    let rpc_url = RPC_URL.parse()?;
    let provider = ProviderBuilder::new().on_http(rpc_url);

    // Get latest block number
    // let latest_block: BlockNumberOrTag = indexer::get_latest_block_number(&provider).await?;
    // let latest_block: BlockNumberOrTag = BlockNumberOrTag::Number(10000000);
    let latest_block: BlockNumberOrTag = BlockNumberOrTag::Number(21319680);

    println!("Latest block number: {:?}", latest_block);

    // Get block by number
    let kind = BlockTransactionsKind::Full; // Hashes: only include tx hashes, Full: include full tx objects
    let block = indexer::get_block_by_number(&provider, latest_block, kind)
        .await?
        .ok_or_else(|| eyre::eyre!("Provider returned no block"))?;

    // println!("Block: {:?}", block);

    // Get receipts by block number
    let block_id = BlockId::Number(latest_block);
    let receipts = indexer::get_block_receipts(&provider, block_id)
        .await?
        .ok_or_else(|| eyre::eyre!("Provider returned no receipts"))?;

    // println!("Receipts: {:?}", receipts);

    // Parse all block data
    let parsed_data = indexer::parse_data(block, receipts).await?; 
    // println!("Parsed data: {:?}", parsed_data);
    
    let transformed_data = indexer::transform_data(parsed_data).await?;
    println!("Transformed data: {:?}", transformed_data);

    Ok(())
}