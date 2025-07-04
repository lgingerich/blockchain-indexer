mod indexer;
mod metrics;
mod models;
mod storage;
mod utils;

use alloy_eips::BlockNumberOrTag;
use alloy_network::AnyNetwork;
use alloy_provider::RootProvider;
use alloy_rpc_client::RpcClient;
use alloy_transport_http::Http;
use anyhow::Result;
use futures::{stream::FuturesUnordered, StreamExt};
use http::{HeaderMap, HeaderValue};
use reqwest;
use std::{future::Future, pin::Pin};
use tokio::{signal, time::Instant};
use tracing::{error, info};
use tracing_subscriber::{self, EnvFilter};
use url::Url;

use crate::metrics::Metrics;
use crate::models::common::{Chain, TransformedData};
use crate::models::datasets::blocks::TransformedBlockData;
use crate::storage::{setup_channels, DatasetType};
use crate::utils::load_config;

const SLEEP_DURATION: u64 = 3000; // 3000 ms = 3s
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
    let config = load_config("config.yml")?;
    info!("Config loaded successfully");

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
    let dataset_location = config.dataset_location;

    // Initialize optional metrics
    let metrics: Option<Metrics> = if metrics_enabled {
        Some(Metrics::new(chain_name.to_string())?)
    } else {
        info!("Metrics are disabled");
        None
    };

    // Start metrics server if metrics are enabled
    if let Some(metrics_instance) = &metrics {
        let _ = metrics_instance
            .start_metrics_server(metrics_addr.as_str(), metrics_port)
            .await;
    }

    // Create RPC provider with no-cache headers to ensure we always get fresh data
    // This prevents any potential caching issues that could lead to stale data
    let rpc_url: Url = rpc.parse()?;
    info!("RPC URL: {:?}", rpc_url);

    // Create HTTP client with no-cache headers
    let mut headers = HeaderMap::new();
    headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
    let http = Http::with_client(
        reqwest::Client::builder()
            .default_headers(headers)
            .build()?,
        rpc_url,
    );

    // Create RPC client and provider
    let rpc_client = RpcClient::new(http, true);
    let provider: RootProvider<AnyNetwork> = RootProvider::new(rpc_client);

    // Get chain ID
    let chain_id = indexer::get_chain_id(&provider, metrics.as_ref()).await?;
    let chain = Chain::from_chain_id(chain_id)?;
    info!("Chain ID: {:?}", chain_id);

    // Set up channels
    let channels = setup_channels(chain_name.as_str(), metrics.as_ref()).await?;

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

    // Create dataset and tables. Ensure everything is ready before proceeding.
    storage::initialize_storage(chain_name.as_str(), &dataset_location, &datasets, chain).await?;

    // Get last processed block number from storage
    let last_processed_block =
        storage::bigquery::get_last_processed_block(chain_name.as_str(), &datasets).await?;

    // Use the maximum of last_processed_block + 1 and start_block (if specified)
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

    // Convert block number to proper type
    let mut block_number_to_process = BlockNumberOrTag::Number(block_number);

    // Get initial latest block number before loop
    let latest_block_tag = indexer::get_latest_block_number(&provider, metrics.as_ref()).await?;
    let mut last_known_latest_block = latest_block_tag.as_number().ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid block number response: {}",
            format!("{:?}", latest_block_tag)
        )
    })?;

    info!("Initial chain tip: Block {}", last_known_latest_block);
    info!(
        "Starting indexing from block {} (distance from tip: {} blocks)",
        block_number,
        last_known_latest_block.saturating_sub(block_number)
    );

    println!();
    info!("========================= STARTING INDEXER =========================");

    let start_time = Instant::now();
    let mut last_metric_update = Instant::now();
    let mut blocks_since_last_metric = 0;

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
            anyhow::anyhow!(
                "Invalid block number response: {}",
                format!("{:?}", block_number_to_process)
            )
        })? > last_known_latest_block.saturating_sub(chain_tip_buffer * 2)
        {
            let latest_block: BlockNumberOrTag =
                indexer::get_latest_block_number(&provider, metrics.as_ref()).await?;

            last_known_latest_block = latest_block.as_number().ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid block number response: {}",
                    format!("{:?}", latest_block)
                )
            })?;
        }

        // If indexer gets too close to tip, back off and retry
        if last_known_latest_block.saturating_sub(block_number_to_process.as_number().ok_or_else(
            || {
                anyhow::anyhow!(
                    "Invalid block number response: {}",
                    format!("{:?}", block_number_to_process)
                )
            },
        )?) < chain_tip_buffer
        {
            info!(
                "Buffer limit reached. Waiting for current block to be {} blocks behind tip: {} - current distance: {} - sleeping for {} seconds",
                chain_tip_buffer,
                last_known_latest_block,
                last_known_latest_block.saturating_sub(block_number_to_process.as_number().ok_or_else(|| anyhow::anyhow!("Invalid block number response: {}", format!("{:?}", block_number_to_process)))?),
                SLEEP_DURATION as f64 / 1000.0
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(SLEEP_DURATION)).await;
            continue;
        }

        // Calculate how many blocks we can process in this batch
        let blocks_to_process = if let Some(end) = end_block {
            let remaining = end.saturating_sub(block_number).saturating_add(1);
            BATCH_SIZE.min(remaining as usize)
        } else {
            BATCH_SIZE
        };

        // Create a batch of block numbers to process
        let block_batch: Vec<u64> = (0..blocks_to_process)
            .map(|i| block_number + i as u64)
            .collect();

        // Process blocks using Tokio's built-in parallelism
        let futures: FuturesUnordered<
            Pin<Box<dyn Future<Output = Result<TransformedData>> + Send>>,
        > = FuturesUnordered::new();

        for block_num in &block_batch {
            let provider = provider.clone();
            let datasets = datasets.clone();
            let metrics_ref = metrics.as_ref();

            futures.push(Box::pin(async move {
                let block_start_time = Instant::now();

                // Process the block
                let result = indexer::process_block(
                    &provider,
                    BlockNumberOrTag::Number(*block_num),
                    chain,
                    chain_id,
                    &datasets,
                    metrics_ref,
                )
                .await;

                // Update metrics for this block if available
                if let Some(metrics_instance) = metrics_ref {
                    metrics_instance.record_blocks_processed(1);
                    metrics_instance.record_latest_processed_block(*block_num);
                    metrics_instance.record_latest_block_processing_time(
                        block_start_time.elapsed().as_secs_f64(),
                    );
                    metrics_instance.record_chain_tip(last_known_latest_block);
                    metrics_instance
                        .record_chain_tip_lag(last_known_latest_block.saturating_sub(*block_num));
                }

                result
            }));
        }

        let results: Vec<Result<TransformedData>> = futures.collect().await;

        // Process and save results
        // Check if any L1 batch data is missing and retry if so
        let mut first_unavailable_block: Option<u64> = None;
        let mut successful_blocks = 0;

        for (block_num, result) in block_batch.iter().zip(results) {
            match result {
                Ok(transformed_data) => {
                    // Check for ZKSync L1 batch number availability
                    if let Some(TransformedBlockData::ZKsync(zk_block)) =
                        transformed_data.blocks.first()
                    {
                        if zk_block.l1_batch_number.is_none() {
                            info!(
                                "L1 batch number not yet available for block {}. Will retry this and subsequent blocks.",
                                block_num
                            );
                            first_unavailable_block = Some(*block_num);
                            break; // Exit the loop - no point processing further blocks
                        }
                    }

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
                            let _ = channels.send_dataset(dataset, *block_num).await;
                        }
                    }

                    info!(
                        "Successfully processed and queued block {} for storage",
                        block_num
                    );
                    successful_blocks += 1;
                }
                Err(e) => {
                    // This is an unrecoverable error that survived all retries in mod.rs
                    return Err(anyhow::anyhow!(
                        "Fatal error processing block {}: {}",
                        block_num,
                        e
                    ));
                }
            }
        }

        // If we found any block with missing L1 batch number
        if let Some(unavailable_block) = first_unavailable_block {
            info!(
                "Waiting for L1 batch number to become available for block {}",
                unavailable_block
            );
            // Sleep before retrying
            tokio::time::sleep(tokio::time::Duration::from_millis(SLEEP_DURATION)).await;
            // Set block_number to the first unavailable block
            block_number = unavailable_block;
            block_number_to_process = BlockNumberOrTag::Number(block_number);
            continue;
        } else {
            // Only increment by the number of successfully processed blocks if we didn't find any unavailable blocks
            block_number += successful_blocks as u64;
            block_number_to_process = BlockNumberOrTag::Number(block_number);
        }

        if let Some(metrics_instance) = &metrics {
            // Update blocks processed count
            blocks_since_last_metric += successful_blocks;

            // Update blocks per second every second
            let elapsed = last_metric_update.elapsed();
            if elapsed.as_secs() >= 1 {
                let blocks_per_second = blocks_since_last_metric as f64 / elapsed.as_secs_f64();
                metrics_instance.record_blocks_per_second(blocks_per_second);

                // Reset counters
                blocks_since_last_metric = 0;
                last_metric_update = Instant::now();
            }
        }
    }
}
