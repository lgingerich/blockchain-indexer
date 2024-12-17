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
use tracing::{info, warn, error};
use tracing_subscriber::{self, EnvFilter};

use crate::models::indexed::blocks::TransformedBlockData;
use crate::models::indexed::transactions::TransformedTransactionData;
use crate::models::indexed::logs::TransformedLogData;
use crate::models::indexed::traces::TransformedTraceData;

const RPC_URL: &str = "https://eth.drpc.org";
// TODO: Tenderly RPC throws errors for some blocks (e.g. 15_000_000)
// const RPC_URL: &str = "https://mainnet.era.zksync.io";
const MAX_BATCH_SIZE: usize = 10;

// TODO: Make datasets optional as some will be empty in early chain history
// handle error when provider does not return data

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive(tracing::Level::INFO.into()))
        .init();

    // Create dataset and tables
    let dataset_id = "test_dataset";
    let result_dataset = storage::bigquery::create_dataset(dataset_id).await;
    for table in ["blocks", "logs", "transactions", "traces"] {
        let result_table = storage::bigquery::create_table(dataset_id, table).await;
    }

    // Create a RPC provider using HTTP with the `reqwest` crate
    let rpc_url = RPC_URL.parse()?;
    let provider = ProviderBuilder::new().on_http(rpc_url);

    //////////////////////// Fetch data ////////////////////////
    let chain_id = indexer::get_chain_id(&provider).await?;
    info!("Chain ID: {:?}", chain_id);

    // Initialize data for loop
    let mut block_number: u64 = 15_000_000;
    let mut block_number_to_process = BlockNumberOrTag::Number(block_number);
    let mut blocks_collection: Vec<TransformedBlockData> = vec![];
    let mut transactions_collection: Vec<TransformedTransactionData> = vec![];
    let mut logs_collection: Vec<TransformedLogData> = vec![];
    let mut traces_collection: Vec<TransformedTraceData> = vec![];

    loop  {
        // // Get latest block number
        // let latest_block: BlockNumberOrTag = indexer::get_latest_block_number(&provider).await?;

        info!("Block number to process: {:?}", block_number_to_process);

        // Get block by number
        let kind = BlockTransactionsKind::Full; // Hashes: only include tx hashes, Full: include full tx objects
        let block = indexer::get_block_by_number(&provider, block_number_to_process, kind)
            .await?
            .ok_or_else(|| eyre::eyre!("Provider returned no block"))?;

        // Get receipts by block number
        let block_id = BlockId::Number(block_number_to_process);
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
        let traces = indexer::debug_trace_block_by_number(&provider, block_number_to_process, trace_options).await?;

        // Extract and separate the raw RPC response into distinct datasets (block headers, transactions, withdrawals, receipts, logs, traces)
        let parsed_data = indexer::parse_data(chain_id, block, receipts, traces).await?; 

        // Transform all data into final output formats (blocks, transactions, logs, traces)
        let transformed_data = indexer::transform_data(parsed_data).await?;

        blocks_collection.extend(transformed_data.blocks);  
        transactions_collection.extend(transformed_data.transactions);
        logs_collection.extend(transformed_data.logs); // TODO: block_timestamp is None for some (or all) logs
        traces_collection.extend(transformed_data.traces);

        if blocks_collection.len() >= MAX_BATCH_SIZE {
            // Insert data into BigQuery
            // This waits for each dataset to be inserted before inserting the next one
            // TODO: Add parallel insert
            storage::bigquery::insert_data("test_dataset", "blocks", blocks_collection).await?;
            storage::bigquery::insert_data("test_dataset", "transactions", transactions_collection).await?;
            storage::bigquery::insert_data("test_dataset", "logs", logs_collection).await?;
            storage::bigquery::insert_data("test_dataset", "traces", traces_collection).await?;

            // Reset collections
            blocks_collection = vec![];
            transactions_collection = vec![];
            logs_collection = vec![];
            traces_collection = vec![];
        }

        // Increment the raw number and update BlockNumberOrTag
        block_number += 1;
        block_number_to_process = BlockNumberOrTag::Number(block_number);
    }
}