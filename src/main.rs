mod indexer;
mod metrics;
mod models;
mod storage;
mod utils;

use alloy_consensus::TxEnvelope;
use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_network::{primitives::BlockTransactionsKind, AnyNetwork, AnyTxEnvelope};
use alloy_primitives::FixedBytes;
use alloy_provider::ProviderBuilder;
use alloy_rpc_types_trace::geth::{
    GethDebugBuiltInTracerType, GethDebugTracerConfig, GethDebugTracerType,
    GethDebugTracingOptions, GethDefaultTracingOptions,
};
use anyhow::{anyhow, Result};
use opentelemetry::KeyValue;
use rayon::prelude::*;
use tokio::{signal, time::Instant};
use tracing::{error, info};
use tracing_subscriber::{self, EnvFilter};
use url::Url;

use crate::metrics::Metrics;
use crate::models::common::{Chain, TransformedData};
use crate::models::datasets::blocks::RpcHeaderData;
use crate::models::errors::RpcError;
use crate::storage::{setup_channels, DatasetType};
use crate::utils::load_config;

const SLEEP_DURATION: u64 = 1000; // ms
const BATCH_SIZE: usize = 10; // Number of blocks to process in parallel

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
            info!("{:?}", config);
            config
        }
        Err(e) => {
            error!("Failed to load config: {}", e);
            return Err(anyhow!(e));
        }
    };

    // Parse configs
    let chain_name = config.chain_name;
    let start_block = config.start_block;
    let end_block = config.end_block;
    let chain_tip_buffer = config.chain_tip_buffer;
    let rpc = config.rpc_url.as_str();
    let datasets = config.datasets;
    let metrics_enabled = config.metrics.enabled;
    let metrics_addr = config.metrics.address;
    let metrics_port = config.metrics.port;


    // Initialize optional metrics
    let metrics = if metrics_enabled {
        Some(Metrics::new(chain_name.to_string())?)
    } else {
        info!("Metrics are disabled");
        None
    };

    // Start metrics server if metrics are enabled
    if let Some(metrics_instance) = &metrics {
        metrics_instance
            .start_metrics_server(metrics_addr.as_str(), metrics_port)
            .await;
    }

    // Track which RPC responses we need
    let need_block =
        datasets.contains(&"blocks".to_string()) || datasets.contains(&"transactions".to_string()); // Blocks and transactions are dependendent on eth_getBlockByNumber
    let need_receipts =
        datasets.contains(&"logs".to_string()) || datasets.contains(&"transactions".to_string()); // Logs and transactions are dependendent on eth_getBlockReceipts
    let need_traces = datasets.contains(&"traces".to_string()); // Traces are dependendent on eth_debug_traceBlockByNumber

    
    // Create RPC provider
    let rpc_url: Url = rpc.parse()?;
    info!("RPC URL: {:?}", rpc);
    let provider = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .on_http(rpc_url);

    // Get chain ID
    let chain_id = indexer::get_chain_id(&provider, metrics.as_ref()).await?;
    let chain = Chain::from_chain_id(chain_id)?;
    info!("Chain ID: {:?}", chain_id);

    // Create dataset and tables. Ensure everything is ready before proceeding.
    storage::initialize_storage(chain_name.as_str(), &datasets, chain).await?;

    // Set up channels
    let channels = setup_channels(chain_name.as_str()).await?;

    // Create a shutdown signal handler. Flush channels before shutting down.
    let mut shutdown_signal = channels.shutdown_signal();
    let shutdown_channels = channels.clone();
    tokio::spawn(async move {
        if let Ok(()) = signal::ctrl_c().await {
            info!("Received Ctrl+C signal, initiating shutdown...");
            if let Err(e) = shutdown_channels.shutdown(None).await {
                error!("Error during shutdown: {}", e);
            }
        }
    });

    // Get last processed block number from storage
    // Use the maximum of last_processed_block + 1 and start_block (if specified)
    let last_processed_block =
        storage::bigquery::get_last_processed_block(chain_name.as_str(), &datasets).await?;
    let mut block_number = if last_processed_block > 0 {
        // If we have processed blocks, start from the next one
        let next_block = last_processed_block + 1;
        // If start_block is specified, use the maximum of next_block and start_block
        if let Some(start) = start_block {
            next_block.max(start)
        } else {
            next_block
        }
    } else {
        // If no blocks processed yet, use start_block or 0
        start_block.unwrap_or(0)
    };

    // Check if the starting block is already beyond the end block
    if let Some(end) = end_block {
        if block_number > end {
            info!(
                "Starting block number {} is greater than end block {}, nothing to process.",
                block_number, end
            );
            return Ok(());
        }
    }

    info!("Starting block number: {:?}", block_number);

    // Initialize data for loop
    let mut block_number_to_process = BlockNumberOrTag::Number(block_number);

    // Get initial latest block number before loop
    let mut last_known_latest_block =
        indexer::get_latest_block_number(&provider, metrics.as_ref())
            .await?
            .as_number()
            .ok_or_else(|| RpcError::InvalidBlockNumberResponse {
                got: block_number_to_process.to_string(),
            })?;

    println!();
    info!("========================= STARTING INDEXER =========================");

    let start_time = Instant::now();

    loop {
        // Check for shutdown signal (non-blocking)
        if shutdown_signal.try_recv().is_ok() {
            info!("Shutting down main processing loop...");
            // Ensure all channels are flushed before breaking
            channels.shutdown(None).await?;
            break Ok(());
        }

        // Check if we've reached the end block (if specified) before processing
        if let Some(end) = end_block {
            if block_number > end {
                info!(
                    "Reached end block {}, waiting for channels to flush...",
                    end
                );
                // Pass the end block to shutdown so it can verify completion
                channels.shutdown(Some(end)).await?;
                info!("All channels flushed, shutting down.");
                let total_runtime = start_time.elapsed();
                info!("Total runtime: {:.2?}", total_runtime);
                info!(
                    "Blocks processed per second: {:.2?}",
                    (end_block.unwrap_or(0) as f64 - start_block.unwrap_or(0) as f64)
                        / total_runtime.as_secs_f64()
                );
                break Ok(());
            }
        }

        // Only check latest block if we're within 2x buffer of last known tip
        if block_number_to_process.as_number().ok_or_else(|| {
            RpcError::InvalidBlockNumberResponse {
                got: block_number_to_process.to_string(),
            }
        })? > (last_known_latest_block - chain_tip_buffer * 2)
        {
            let latest_block: BlockNumberOrTag =
                indexer::get_latest_block_number(&provider, metrics.as_ref())
                    .await?;

            last_known_latest_block =
                latest_block
                    .as_number()
                    .ok_or_else(|| RpcError::InvalidBlockNumberResponse {
                        got: latest_block.to_string(),
                    })?;
        }

        // If indexer gets too close to tip, back off and retry
        if block_number_to_process.as_number().unwrap()
            > (last_known_latest_block - chain_tip_buffer)
        {
            info!(
                "Buffer limit reached. Waiting for current block to be {} blocks behind tip: {} - current distance: {} - sleeping for 1s",
                chain_tip_buffer,
                last_known_latest_block,
                last_known_latest_block - block_number_to_process.as_number().unwrap()
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(SLEEP_DURATION)).await;
            continue;
        }

        // Check channel capacity and apply backpressure if needed
        while !channels.check_capacity(metrics.as_ref()).await? {
            info!(
                "Applying backpressure - sleeping for {} seconds...",
                SLEEP_DURATION / 1000
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(SLEEP_DURATION)).await;
        }

        // Calculate how many blocks we can process in this batch
        let blocks_to_process = if let Some(end) = end_block {
            let remaining = end - block_number + 1;
            BATCH_SIZE.min(remaining as usize)
        } else {
            BATCH_SIZE
        };

        // Create a batch of block numbers to process
        let block_batch: Vec<u64> = (0..blocks_to_process)
            .map(|i| block_number + i as u64)
            .collect();

        // Process blocks in parallel using Rayon
        let results: Vec<Result<TransformedData>> = block_batch.par_iter().map(|&block_num| {
            let block_start_time = Instant::now();
            
            // Process the block
            let result = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(indexer::process_block(
                    &provider,
                    BlockNumberOrTag::Number(block_num),
                    chain,
                    chain_id,
                    &datasets,
                    metrics.as_ref(),
                ));

            // Update metrics for this block
            if let Some(metrics_instance) = &metrics {
                metrics_instance.blocks_processed.add(
                    1,
                    &[KeyValue::new("chain", metrics_instance.chain_name.clone())],
                );
                metrics_instance.latest_processed_block.record(
                    block_num,
                    &[KeyValue::new("chain", metrics_instance.chain_name.clone())],
                );
                metrics_instance.latest_block_processing_time.record(
                    block_start_time.elapsed().as_secs_f64(),
                    &[KeyValue::new("chain", metrics_instance.chain_name.clone())],
                );
                metrics_instance.chain_tip_block.record(
                    last_known_latest_block,
                    &[KeyValue::new("chain", metrics_instance.chain_name.clone())],
                );
                metrics_instance.chain_tip_lag.record(
                    last_known_latest_block - block_num,
                    &[KeyValue::new("chain", metrics_instance.chain_name.clone())],
                );
            }

            result
        }).collect();

        // Handle results and send to channels
        for (block_num, result) in block_batch.iter().zip(results) {
            match result {
                Ok(transformed_data) => {
                    // Send transformed data through channels for saving to storage
                    let dataset_mappings = [
                        ("blocks", DatasetType::Blocks(transformed_data.blocks)),
                        (
                            "transactions",
                            DatasetType::Transactions(transformed_data.transactions),
                        ),
                        ("logs", DatasetType::Logs(transformed_data.logs)),
                        ("traces", DatasetType::Traces(transformed_data.traces)),
                    ];

                    for (dataset_name, dataset) in dataset_mappings {
                        if datasets.contains(&dataset_name.to_string()) {
                            channels
                                .send_dataset(dataset, *block_num)
                                .await;
                        }
                    }

                    info!("Finished processing block {}", block_num);
                }
                Err(e) => {
                    error!("Error processing block {}: {}", block_num, e);
                }
            }
        }

        // Increment the block number by the batch size
        block_number += blocks_to_process as u64;
        block_number_to_process = BlockNumberOrTag::Number(block_number);
    }
}
