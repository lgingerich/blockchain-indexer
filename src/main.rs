mod indexer;
mod metrics;
mod models;
mod storage;
mod utils;

use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_network::{primitives::BlockTransactionsKind, AnyNetwork};
use alloy_provider::ProviderBuilder;
use alloy_rpc_types_trace::geth::{
    GethDebugBuiltInTracerType, GethDebugTracerConfig, GethDebugTracerType,
    GethDebugTracingOptions, GethDefaultTracingOptions,
};
use anyhow::{anyhow, Result};
use opentelemetry::KeyValue;
use tokio::{
    signal,
    time::Instant,
};
use tracing::{error, info};
use tracing_subscriber::{self, EnvFilter};
use url::Url;

use crate::metrics::Metrics;
use crate::models::common::Chain;
use crate::models::datasets::blocks::RpcHeaderData;
use crate::storage::setup_channels;
use crate::utils::{hex_to_u64, load_config};

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
            config
        }
        Err(e) => {
            error!("Failed to load config: {}", e);
            return Err(anyhow!(e));
        }
    };

    // Parse configs
    let dataset_id = config.project_name.as_str();
    let chain_name = config.chain_name.to_owned(); // Create owned String to pass to metrics
    let chain_id = config.chain_id;
    let chain_tip_buffer = config.chain_tip_buffer;
    let rpc = config.rpc_url.as_str();
    let datasets = config.datasets;
    let metrics_enabled = config.metrics.enabled;

    let chain = Chain::from_chain_id(chain_id);

    // Initialize optional metrics
    let metrics = if metrics_enabled {
        Some(Metrics::new(chain_name.to_string())?)
    } else {
        info!("Metrics are disabled");
        None
    };

    // Start metrics server if metrics are enabled
    if let Some(metrics_instance) = &metrics {
        metrics_instance.start_metrics_server("0.0.0.0", 9100).await; // Prometheus port is currently hardcoded to 9100 in prometheus.yml
    }

    // Track which RPC responses we need
    let need_block =
        datasets.contains(&"blocks".to_string()) || datasets.contains(&"transactions".to_string()); // Blocks and transactions are dependendent on eth_getBlockByNumber
    let need_receipts =
        datasets.contains(&"logs".to_string()) || datasets.contains(&"transactions".to_string()); // Logs and transactions are dependendent on eth_getBlockReceipts
    let need_traces = datasets.contains(&"traces".to_string()); // Traces are dependendent on eth_debug_traceBlockByNumber

    // Set up channels
    let channels = setup_channels(dataset_id).await?;

    // Create a shutdown signal handler. Flush channels before shutting down.
    let mut shutdown_signal = channels.shutdown_signal();
    let shutdown_channels = channels.clone();
    tokio::spawn(async move {
        if let Ok(()) = signal::ctrl_c().await {
            info!("Received Ctrl+C signal, initiating shutdown...");
            if let Err(e) = shutdown_channels.shutdown().await {
                error!("Error during shutdown: {}", e);
            }
        }
    });

    // Create dataset and tables. Handles existing datasets and tables.
    let _ = storage::bigquery::create_dataset_with_retry(dataset_id).await;
    for table in ["blocks", "logs", "transactions", "traces"] {
        if datasets.contains(&table.to_string()) {
            let _ = storage::bigquery::create_table_with_retry(dataset_id, table, chain).await;
        }
    }

    // Get last processed block number from storage
    // If it exists, start from the next block, else start from 0
    let last_processed_block =
        storage::bigquery::get_last_processed_block(dataset_id, &datasets).await?;
    let mut block_number = if last_processed_block > 0 {
        last_processed_block + 1
    } else {
        0
    };

    info!("Starting block number: {:?}", block_number);

    // Create RPC provider
    let rpc_url: Url = rpc.parse()?;
    info!("RPC URL: {:?}", rpc);
    let provider = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .on_http(rpc_url);

    // Get chain ID
    let chain_id = indexer::get_chain_id(&provider, metrics.as_ref()).await?;
    info!("Chain ID: {:?}", chain_id);

    // Initialize data for loop
    let mut block_number_to_process = BlockNumberOrTag::Number(block_number);

    println!();
    info!("========================= STARTING INDEXER =========================");
    
    loop {
        // Check for shutdown signal (non-blocking)
        if shutdown_signal.try_recv().is_ok() {
            info!("Shutting down main processing loop...");
            break Ok(());
        }

        // Initialize intermediate data
        let mut block = None;
        let mut receipts = None;
        let mut traces = None;

        // Get latest block number
        // Note: Since the indexer is not real-time, this never gets used other than to check if we're too close to the tip
        let latest_block: BlockNumberOrTag =
            indexer::get_latest_block_number(&provider, metrics.as_ref()).await?;

        info!("Block number to process: {:?}", block_number_to_process);

        // If indexer gets too close to tip, back off and retry
        // Note: Real-time processing is not implemented
        if block_number_to_process.as_number().unwrap()
            > (latest_block.as_number().unwrap() - chain_tip_buffer)
        {
            info!(
                "Buffer limit reached. Waiting for current block to be {} blocks behind tip: {:?} — current distance: {:?} - sleeping for 1s",
                chain_tip_buffer,
                hex_to_u64(latest_block.to_string()).unwrap(),
                (hex_to_u64(latest_block.to_string()).unwrap() - hex_to_u64(block_number_to_process.to_string()).unwrap())
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(SLEEP_DURATION)).await;
            continue;
        }
        
        // Check channel capacity and apply backpressure if needed
        while !channels.check_capacity(metrics.as_ref()).await? {
            info!("Applying backpressure - sleeping for {} seconds...", SLEEP_DURATION / 1000);
            tokio::time::sleep(tokio::time::Duration::from_millis(SLEEP_DURATION)).await;
        }

        // Start timing the block processing
        let block_start_time = Instant::now();

        // Get block by number
        // Only fetch block data if `blocks` or `transactions` are in the active datasets
        if need_block {
            let kind = BlockTransactionsKind::Full; // Hashes: only include tx hashes, Full: include full tx objects
            block = Some(
                indexer::get_block_by_number(&provider, block_number_to_process, kind, metrics.as_ref())
                    .await?
                    .ok_or_else(|| anyhow!("Provider returned no block"))?,
            );
        }

        // Get receipts by block number
        // Only fetch receipts data if `logs` or `transactions` are in the active datasets
        if need_receipts {
            let block_id = BlockId::Number(block_number_to_process);
            receipts = Some(
                indexer::get_block_receipts(&provider, block_id, metrics.as_ref())
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
                    metrics.as_ref(),
                )
                .await?
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

        // Send transformed data through channels for saving to storage
        if datasets.contains(&"blocks".to_string()) {
            if let Err(e) = channels.blocks_tx.send((transformed_data.blocks, block_number_to_process.as_number().unwrap())).await {
                error!("Failed to send blocks batch to channel: {}", e);
            }
        }
        
        if datasets.contains(&"transactions".to_string()) {
            if let Err(e) = channels.transactions_tx.send((transformed_data.transactions, block_number_to_process.as_number().unwrap())).await {
                error!("Failed to send transactions batch to channel: {}", e);
            }
        }
        
        if datasets.contains(&"logs".to_string()) {
            if let Err(e) = channels.logs_tx.send((transformed_data.logs, block_number_to_process.as_number().unwrap())).await {
                error!("Failed to send logs batch to channel: {}", e);
            }
        }
        
        if datasets.contains(&"traces".to_string()) {
            if let Err(e) = channels.traces_tx.send((transformed_data.traces, block_number_to_process.as_number().unwrap())).await {
                error!("Failed to send traces batch to channel: {}", e);
            }
        }

        // Calculate block processing duration
        let block_processing_duration = block_start_time.elapsed().as_secs_f64();

        // Update metrics
        if let Some(metrics_instance) = &metrics {
            metrics_instance.blocks_processed
                .add(1, &[KeyValue::new("chain", metrics_instance.chain_name.clone())]);
            metrics_instance.latest_processed_block.record(
                block_number_to_process.as_number().unwrap(),
                &[KeyValue::new("chain", metrics_instance.chain_name.clone())],
            );
            metrics_instance.latest_block_processing_time.record(
                block_processing_duration,
                &[KeyValue::new("chain", metrics_instance.chain_name.clone())],
            );
            metrics_instance.chain_tip_block.record(
                latest_block.as_number().unwrap(),
                &[KeyValue::new("chain", metrics_instance.chain_name.clone())],
            );
            metrics_instance.chain_tip_lag.record(
                latest_block.as_number().unwrap() - block_number_to_process.as_number().unwrap(),
                &[KeyValue::new("chain", metrics_instance.chain_name.clone())],
            );
        }

        // Increment the raw number and update BlockNumberOrTag
        block_number += 1;
        block_number_to_process = BlockNumberOrTag::Number(block_number);
    }
}
