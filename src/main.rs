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
use std::sync::Arc;

use crate::metrics::Metrics;
use crate::models::common::Chain;
use crate::models::datasets::blocks::RpcHeaderData;
use crate::models::errors::RpcError;
use crate::storage::{setup_channels, DatasetType};
use crate::utils::load_config;
use crate::utils::rate_limiter::RateLimiter;
use std::time::Duration;

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
    
    // Initialize rate limiter with adaptive concurrency control
    // These values can be adjusted based on the RPC provider's limits and performance
    let rate_limiter = RateLimiter::new(
        10,                              // initial_limit: Start with 10 concurrent requests
        500,                             // max_limit: Maximum of 500 concurrent requests
        50,                              // window_size: Track last 50 requests for adaptation
        Duration::from_millis(200),      // target_response_time: Aim for 200ms response time
        Duration::from_secs(1),          // adaptation_interval: Adjust limits every second
    );
    info!("Rate limiter initialized with initial concurrency limit: {}", rate_limiter.get_current_limit());

    // Add a periodic check of the rate limiter status
    let rate_limiter_for_status = Arc::new(rate_limiter);
    let rate_limiter_status_clone = Arc::clone(&rate_limiter_for_status);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            info!("Rate limiter status - current limit: {}", rate_limiter_status_clone.get_current_limit());
        }
    });

    // Use the Arc-wrapped rate limiter for the rest of the code
    let rate_limiter = rate_limiter_for_status;

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
    let chain_id = indexer::get_chain_id(&provider, metrics.as_ref(), Some(&rate_limiter)).await?;
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
            info!("Starting block number {} is greater than end block {}, nothing to process.", block_number, end);
            return Ok(());
        }
    }

    info!("Starting block number: {:?}", block_number);

    // Initialize data for loop
    let mut block_number_to_process = BlockNumberOrTag::Number(block_number);

    // Get initial latest block number before loop
    let mut last_known_latest_block = indexer::get_latest_block_number(&provider, metrics.as_ref(), Some(&rate_limiter))
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
            // Shutdown rate limiter
            rate_limiter.shutdown();
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
                // Shutdown rate limiter
                rate_limiter.shutdown();
                info!("All channels flushed, shutting down.");
                let total_runtime = start_time.elapsed();
                info!("Total runtime: {:.2?}", total_runtime);
                info!("Blocks processed per second: {:.2?}", (end_block.unwrap_or(0) as f64 - start_block.unwrap_or(0) as f64) / total_runtime.as_secs_f64());
                break Ok(());
            }
        }

        // Initialize intermediate data
        let mut block = None;
        let mut receipts = None;
        let mut traces = None;

        // Only check latest block if we're within 2x buffer of last known tip
        if block_number_to_process.as_number().ok_or_else(|| {
            RpcError::InvalidBlockNumberResponse {
                got: block_number_to_process.to_string(),
            }
        })? > (last_known_latest_block - chain_tip_buffer * 2)
        {
            let latest_block: BlockNumberOrTag =
                indexer::get_latest_block_number(&provider, metrics.as_ref(), Some(&rate_limiter)).await?;
                
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

        // Start timing the block processing
        let block_start_time = Instant::now();

        // Get block by number
        // Only fetch block data if `blocks` or `transactions` are in the active datasets
        if need_block {
            let kind = BlockTransactionsKind::Full; // Hashes: only include tx hashes, Full: include full tx objects
            let block_result = indexer::get_block_by_number(
                &provider,
                block_number_to_process,
                kind,
                metrics.as_ref(),
                Some(&rate_limiter),
            ).await;
            
            block = Some(
                block_result?
                .ok_or_else(|| anyhow!("Provider returned no block"))?,
            );
        }

        // Get receipts by block number
        // Only fetch receipts data if `logs` or `transactions` are in the active datasets
        if need_receipts {
            let block_id = BlockId::Number(block_number_to_process);
            let receipts_result = indexer::get_block_receipts(
                &provider, 
                block_id, 
                metrics.as_ref(),
                Some(&rate_limiter),
            ).await;
            
            receipts = Some(
                receipts_result?
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

            // Get Geth traces by transaction hash
            let tx_hashes: Vec<FixedBytes<32>> = if let Some(block_data) = &block {
                block_data
                    .transactions
                    .txns()
                    .map(|transaction| {
                        match &transaction.inner.inner {
                            AnyTxEnvelope::Ethereum(inner) => {
                                match inner {
                                    TxEnvelope::Legacy(signed) => *signed.hash(),
                                    TxEnvelope::Eip2930(signed) => *signed.hash(),
                                    TxEnvelope::Eip1559(signed) => *signed.hash(),
                                    TxEnvelope::Eip4844(signed) => *signed.hash(),
                                    TxEnvelope::Eip7702(signed) => *signed.hash(),
                                    _ => FixedBytes::<32>::ZERO, // Should never happen
                                }
                            }
                            AnyTxEnvelope::Unknown(unknown) => unknown.hash,
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };
            
            let traces_result = indexer::debug_trace_transaction_by_hash(
                &provider,
                tx_hashes,
                trace_options,
                metrics.as_ref(),
                Some(&rate_limiter),
            ).await;
            
            traces = Some(
                traces_result?
                .ok_or_else(|| anyhow!("Provider returned no traces"))?,
            );
        }

        // Extract and separate the raw RPC response into distinct datasets (block headers, transactions, receipts, logs, traces)
        let parsed_data = indexer::parse_data(
            chain,
            chain_id,
            block_number_to_process.as_number().unwrap(),
            block,
            receipts,
            traces,
        )
        .await?;

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
                    tokio::time::sleep(tokio::time::Duration::from_millis(SLEEP_DURATION)).await;
                    continue;
                }
            }
        }

        // Transform all data into final output formats (blocks, transactions, logs, traces)
        let transformed_data = indexer::transform_data(chain, parsed_data, &datasets).await?;

        info!(
            "Finished processing block {} (concurrency limit: {})",
            block_number_to_process.as_number().unwrap(),
            rate_limiter.get_current_limit()
        );

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
                    .send_dataset(dataset, block_number_to_process.as_number().unwrap())
                    .await;
            }
        }

        // Calculate block processing duration
        let block_processing_duration = block_start_time.elapsed().as_secs_f64();

        // Update metrics
        if let Some(metrics_instance) = &metrics {
            metrics_instance.blocks_processed.add(
                1,
                &[KeyValue::new("chain", metrics_instance.chain_name.clone())],
            );
            metrics_instance.latest_processed_block.record(
                block_number_to_process.as_number().unwrap(),
                &[KeyValue::new("chain", metrics_instance.chain_name.clone())],
            );
            metrics_instance.latest_block_processing_time.record(
                block_processing_duration,
                &[KeyValue::new("chain", metrics_instance.chain_name.clone())],
            );
            metrics_instance.chain_tip_block.record(
                last_known_latest_block,
                &[KeyValue::new("chain", metrics_instance.chain_name.clone())],
            );
            metrics_instance.chain_tip_lag.record(
                last_known_latest_block - block_number_to_process.as_number().unwrap(),
                &[KeyValue::new("chain", metrics_instance.chain_name.clone())],
            );
        }

        // Increment the raw number and update BlockNumberOrTag
        block_number += 1;
        block_number_to_process = BlockNumberOrTag::Number(block_number);
    }
}
