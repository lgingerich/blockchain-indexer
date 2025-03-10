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
use tokio::{signal, time::Instant};
use tracing::{error, info};
use tracing_subscriber::{self, EnvFilter};
use url::Url;
use tokio::task;
use futures::stream::{self, StreamExt};
use std::sync::Arc;

use crate::metrics::Metrics;
use crate::models::common::Chain;
use crate::models::datasets::blocks::RpcHeaderData;
use crate::models::errors::RpcError;
use crate::storage::{setup_channels, DatasetType};
use crate::utils::load_config;

const SLEEP_DURATION: u64 = 1000; // ms

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
    let max_concurrent_blocks = config.max_concurrent_blocks.unwrap_or(10);

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

    // Get initial latest block number before loop
    let mut last_known_latest_block =
        indexer::get_latest_block_number(&provider, metrics.as_ref())
            .await?
            .as_number()
            .ok_or_else(|| RpcError::InvalidBlockNumberResponse {
                got: block_number.to_string(),
            })?;

    println!();
    info!("========================= STARTING INDEXER =========================");

    let start_time = Instant::now();

    // Create block processor
    let block_processor = Arc::new(indexer::block_processor::BlockProcessor::new(
        Box::new(provider),
        chain,
        chain_id,
        metrics.clone(),
        datasets.clone(),
    ));

    // Main processing loop
    loop {
        // Check for shutdown signal (non-blocking)
        if shutdown_signal.try_recv().is_ok() {
            info!("Shutting down main processing loop...");
            // Ensure all channels are flushed before breaking
            channels.shutdown(None).await?;
            break Ok(());
        }

        // Check if we've reached the end block
        if let Some(end) = end_block {
            if block_number > end {
                info!(
                    "Reached end block {}, waiting for channels to flush...",
                    end
                );
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
        if block_number > (last_known_latest_block - chain_tip_buffer * 2) {
            let latest_block = indexer::get_latest_block_number(&provider, metrics.as_ref()).await?;
            last_known_latest_block = latest_block
                .as_number()
                .ok_or_else(|| RpcError::InvalidBlockNumberResponse {
                    got: latest_block.to_string(),
                })?;
        }

        // If indexer gets too close to tip, back off and retry
        if block_number > (last_known_latest_block - chain_tip_buffer) {
            info!(
                "Buffer limit reached. Waiting for current block to be {} blocks behind tip: {} - current distance: {} - sleeping for 1s",
                chain_tip_buffer,
                last_known_latest_block,
                last_known_latest_block - block_number
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

        // Calculate the range of blocks to process in parallel
        let end_block_batch = if let Some(end) = end_block {
            (block_number + max_concurrent_blocks - 1).min(end)
        } else {
            block_number + max_concurrent_blocks - 1
        };

        // Process blocks in parallel
        let block_processor = Arc::clone(&block_processor);
        let channels = channels.clone();
        let block_numbers: Vec<u64> = (block_number..=end_block_batch).collect();

        let results = stream::iter(block_numbers)
            .map(|block_num| {
                let block_processor = Arc::clone(&block_processor);
                task::spawn(async move { block_processor.process_block(block_num).await })
            })
            .buffer_unordered(max_concurrent_blocks as usize)
            .collect::<Vec<_>>()
            .await;

        // Process results and send to storage
        for result in results {
            match result {
                Ok(Ok(processed_block)) => {
                    let block_num = processed_block.block_number;
                    let transformed_data = processed_block.data;

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
                            channels.send_dataset(dataset, block_num).await;
                        }
                    }

                    // Update metrics
                    if let Some(metrics_instance) = &metrics {
                        metrics_instance.blocks_processed.add(
                            1,
                            &[KeyValue::new("chain", metrics_instance.chain_name.clone())],
                        );
                        metrics_instance.latest_processed_block.record(
                            block_num,
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
                }
                Ok(Err(e)) => {
                    error!("Error processing block: {}", e);
                    // Consider implementing retry logic here
                }
                Err(e) => {
                    error!("Task error: {}", e);
                    // Consider implementing retry logic here
                }
            }
        }

        // Update block number for next iteration
        block_number = end_block_batch + 1;
    }
}
