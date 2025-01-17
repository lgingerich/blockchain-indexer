// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_assignments)]

mod indexer;
mod models;
mod storage;
mod utils;

use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_network::{primitives::BlockTransactionsKind, AnyNetwork};
use alloy_provider::ProviderBuilder;
use alloy_rpc_types_eth::{Block, TransactionReceipt};
use alloy_rpc_types_trace::geth::{
    GethDebugBuiltInTracerType, GethDebugTracerConfig, GethDebugTracerType,
    GethDebugTracingOptions, GethDefaultTracingOptions,
};

use anyhow::{anyhow, Result};
use tracing::{error, info, warn};
use tracing_subscriber::{self, EnvFilter};
use url::Url;

use crate::models::common::Chain;
use crate::models::datasets::blocks::{RpcHeaderData, TransformedBlockData};
use crate::models::datasets::logs::TransformedLogData;
use crate::models::datasets::traces::TransformedTraceData;
use crate::models::datasets::transactions::TransformedTransactionData;
use crate::utils::{hex_to_u64, load_config};

// NEXT STEPS:

// - Add monitoring
// - Add data quality checks (schema compliance, missing block detection, duplication detection, etc.)
// - Unit tests
//      - Tests for each tx type for each chain
// - Rate limiting?
// - Docker containerization
// - CI/CD
// - Kubernetes/Helm deployment for production
// - Fix Tenderly RPC

// NOTES:
// - Not sure I should implement RPC rotation. Seems like lots of failure modes.
// - Some fields which are optional are being forced to be defined as mandatory because BQ throws errors on handling none/empty fields


const MAX_BATCH_SIZE: usize = 10; // Number of blocks to fetch before inserting into BigQuery

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    println!();
    info!("=========================== INITIALIZING ===========================");

    // Load config
    let config = match load_config("config.yml") {
        Ok(config) => {
            info!("Config loaded successfully");
            config
        }
        Err(e) => {
            error!("Failed to load config: {}", e);
            return Err(anyhow!(e));
        }
    };

    // Parse configs
    let rpc = config.rpc_url.as_str();
    let dataset_id = config.project_name.as_str();
    let datasets = config.datasets;
    let chain_id = config.chain_id;
    let chain_tip_buffer = config.chain_tip_buffer;

    let chain = Chain::from_chain_id(chain_id);

    // Track which RPC responses we need
    let need_block =
        datasets.contains(&"blocks".to_string()) || datasets.contains(&"transactions".to_string()); // Blocks and transactions are dependendent on eth_getBlockByNumber
    let need_receipts =
        datasets.contains(&"logs".to_string()) || datasets.contains(&"transactions".to_string()); // Logs and transactions are dependendent on eth_getBlockReceipts
    let need_traces = datasets.contains(&"traces".to_string()); // Traces are dependendent on eth_debug_traceBlockByNumber

    // Create dataset and tables. Handles existing datasets and tables.
    let result_dataset = storage::bigquery::create_dataset_with_retry(dataset_id).await;
    for table in ["blocks", "logs", "transactions", "traces"] {
        if datasets.contains(&table.to_string()) {
            let result_table =
                storage::bigquery::create_table_with_retry(dataset_id, table, chain).await;
        }
    };

    // Get last processed block number from storage
    // If it exists, start from the next block, else start from 0
    let last_processed_block =
        storage::bigquery::get_last_processed_block(dataset_id, &datasets).await?;
    let mut block_number = if last_processed_block > 0 {
        last_processed_block + 1
    } else {
        0
    };

    // ZKSYNC
    // Legacy (0): 	1451, 1535
    // DynamicFee (2): 4239, 9239
    // EIP-712 (113):	9073, 9416
    // Priority (255):	2030, 8958
    // 254: 			28679967, 35876713

    // Ethereum
    // Legacy (0): 46147
    // EIP-2930 (1): 12244145
    // DynamicFee (2): 12965001
    // EIP-4844 (3): 19426589

    let mut block_number = 53900157;
    info!("Starting block number: {:?}", block_number);

    // Create RPC provider
    let rpc_url: Url = rpc.parse()?;
    info!("RPC URL: {:?}", rpc);
    let provider = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .on_http(rpc_url);

    // Get chain ID
    let chain_id = indexer::get_chain_id(&provider).await?;
    info!("Chain ID: {:?}", chain_id);

    // Initialize data for loop
    let mut block_number_to_process = BlockNumberOrTag::Number(block_number);
    let mut blocks_collection: Vec<TransformedBlockData> = vec![];
    let mut transactions_collection: Vec<TransformedTransactionData> = vec![];
    let mut logs_collection: Vec<TransformedLogData> = vec![];
    let mut traces_collection: Vec<TransformedTraceData> = vec![];

    println!();
    info!("========================= STARTING INDEXER =========================");
    loop {
        // Initialize intermediate data
        let mut block = None;
        let mut receipts = None;
        let mut traces = None;

        // Get latest block number
        // Note: Since the indexer is not real-time, this never gets used other than to check if we're too close to the tip
        let latest_block: BlockNumberOrTag = indexer::get_latest_block_number(&provider).await?;

        info!("Block number to process: {:?}", block_number_to_process);

        // If indexer gets too close to tip, back off and retry
        // Note: Real-time processing is not implemented
        if block_number_to_process.as_number().unwrap() > (latest_block.as_number().unwrap() - chain_tip_buffer) {
            info!(
                "Buffer limit reached. Waiting for current block to be {} blocks behind tip: {:?} — current distance: {:?} - sleeping for 1s",
                chain_tip_buffer,
                hex_to_u64(latest_block.to_string()).unwrap(),
                (hex_to_u64(latest_block.to_string()).unwrap() - hex_to_u64(block_number_to_process.to_string()).unwrap())
            );
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            continue;
        }

        // Get block by number
        // Only fetch block data if `blocks` or `transactions` are in the active datasets
        if need_block {
            let kind = BlockTransactionsKind::Full; // Hashes: only include tx hashes, Full: include full tx objects
            block = Some(
                indexer::get_block_by_number(&provider, block_number_to_process, kind)
                    .await?
                    .ok_or_else(|| anyhow!("Provider returned no block"))?,
            );
        }

        // Get receipts by block number
        // Only fetch receipts data if `logs` or `transactions` are in the active datasets
        if need_receipts {
            let block_id = BlockId::Number(block_number_to_process);
            receipts = Some(
                indexer::get_block_receipts(&provider, block_id)
                    .await?
                    .ok_or_else(|| anyhow!("Provider returned no receipts"))?,
            );
        }

        // Create tracing options with CallTracer and nested calls
        // Only fetch traces data if `traces` is in the active datasets
        if need_traces {
            let trace_options = GethDebugTracingOptions {
                config: GethDefaultTracingOptions::default(),
                tracer: Some(GethDebugTracerType::BuiltInTracer(
                    GethDebugBuiltInTracerType::CallTracer,
                )),
                tracer_config: GethDebugTracerConfig(serde_json::json!({"onlyTopCall": false})), // Get nested calls
                timeout: Some("10s".to_string()),
            };
            // Get Geth debug traces by block number
            traces = Some(
                indexer::debug_trace_block_by_number(
                    &provider,
                    block_number_to_process,
                    trace_options,
                )
                .await?
                .ok_or_else(|| anyhow!("Provider returned no traces"))?,
            );
        }

        // Extract and separate the raw RPC response into distinct datasets (block headers, transactions, receipts, logs, traces)
        let parsed_data = indexer::parse_data(chain, chain_id, block, receipts, traces).await?;
        
        // For ZKSync, wait until L1 batch number is available
        // This is possibly necessary for other L2s as well
        // Note: For future real-time support, this will need to be improved
        if chain == Chain::ZKsync {
            if let Some(RpcHeaderData::ZKsync(zk_header)) = parsed_data.header.first() {
                if zk_header.l1_batch_number.is_none() {
                    info!(
                        "L1 batch number not yet available for block {}. Waiting...",
                        block_number
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    continue;
                }
            }
        }

        // Transform all data into final output formats (blocks, transactions, logs, traces)
        let transformed_data = indexer::transform_data(chain, parsed_data, &datasets).await?;

        blocks_collection.extend(transformed_data.blocks);
        transactions_collection.extend(transformed_data.transactions);
        logs_collection.extend(transformed_data.logs); // TODO: block_timestamp is None for some (or all) logs
        traces_collection.extend(transformed_data.traces);

        if blocks_collection.len() >= MAX_BATCH_SIZE {
            // Insert data into BigQuery
            // This waits for each dataset to be inserted before inserting the next one
            // TODO: Add parallel insert
            if datasets.contains(&"blocks".to_string()) {
                storage::bigquery::insert_data_with_retry(dataset_id, "blocks", blocks_collection)
                    .await?;
            }
            if datasets.contains(&"transactions".to_string()) {
                storage::bigquery::insert_data_with_retry(
                    dataset_id,
                    "transactions",
                    transactions_collection,
                )
                .await?;
            }
            if datasets.contains(&"logs".to_string()) {
                storage::bigquery::insert_data_with_retry(dataset_id, "logs", logs_collection)
                    .await?;
            }
            if datasets.contains(&"traces".to_string()) {
                storage::bigquery::insert_data_with_retry(dataset_id, "traces", traces_collection)
                    .await?;
            }

            // Reset collections
            blocks_collection = vec![];
            transactions_collection = vec![];
            logs_collection = vec![];
            traces_collection = vec![];
        }

        // Increment the raw number and update BlockNumberOrTag
        block_number += 1;
        block_number_to_process = BlockNumberOrTag::Number(block_number);
        // tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}
