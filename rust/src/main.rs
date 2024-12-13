// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

mod indexer;
mod models;
mod storage;

use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_network::primitives::BlockTransactionsKind;
use alloy_provider::ProviderBuilder;
use alloy_rpc_types_eth::{Block, TransactionReceipt};
use alloy_rpc_types_trace::geth::{GethDebugBuiltInTracerType, GethDebugTracerConfig, GethDebugTracerType, GethDebugTracingOptions, GethDefaultTracingOptions};

use eyre::Result;
use tracing_subscriber::{self, EnvFilter};

const RPC_URL: &str = "https://eth.drpc.org";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive(tracing::Level::INFO.into()))
        .init();

    // Create dataset and tables
    let result_dataset = storage::bigquery::create_dataset("test_dataset").await;
    for table in ["blocks", "logs", "transactions", "traces"] {
        let result_table = storage::bigquery::create_table("test_dataset", table).await;
    }

    // Create a RPC provider using HTTP with the `reqwest` crate
    let rpc_url = RPC_URL.parse()?;
    let provider = ProviderBuilder::new().on_http(rpc_url);

    //////////////////////// Fetch data ////////////////////////
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

    // println!("Block: {:?}", block.clone());

    // Get receipts by block number
    let block_id = BlockId::Number(latest_block);
    let receipts = indexer::get_block_receipts(&provider, block_id)
        .await?
        .ok_or_else(|| eyre::eyre!("Provider returned no receipts"))?;

    // Create tracing options with CallTracer and nested calls
    let trace_options = GethDebugTracingOptions {
        config: GethDefaultTracingOptions::default(),
        tracer: Some(GethDebugTracerType::BuiltInTracer(
            GethDebugBuiltInTracerType::CallTracer,
        )),
        tracer_config: GethDebugTracerConfig(serde_json::json!({"onlyTopCall": false})), // Get nested calls
        timeout: Some("10s".to_string()),
    };
    // Get Geth debug traces by block number
    let traces = indexer::debug_trace_block_by_number(&provider, latest_block, trace_options).await?;

    // Extract and separate the raw RPC response into distinct datasets (block headers, transactions, withdrawals, receipts, logs, traces)
    let parsed_data = indexer::parse_data(block, receipts, traces).await?; 

    // Transform all data into final output formats (blocks, transactions, logs, traces)
    let transformed_data = indexer::transform_data(parsed_data).await?;

    // Combine collections of data from multiple blocks
    let mut blocks_collection = vec![];
    let mut transactions_collection = vec![];
    let mut logs_collection = vec![];
    let mut traces_collection = vec![];

    blocks_collection.extend(transformed_data.blocks);  
    transactions_collection.extend(transformed_data.transactions);
    logs_collection.extend(transformed_data.logs); // TODO: block_timestamp is None for some (or all) logs
    traces_collection.extend(transformed_data.traces);

    // println!("Blocks: {:?}", blocks_collection);

    // TODO: This is not working.
    let insert = storage::bigquery::insert_data("test_dataset", "blocks", blocks_collection).await;
    
    Ok(())
}