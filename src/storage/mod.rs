pub mod bigquery;

use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Sender};
use tokio::time::{Duration, Instant};
use tracing::{debug, error, info};

use crate::metrics::Metrics;
use crate::models::common::Chain;
use crate::models::datasets::blocks::TransformedBlockData;
use crate::models::datasets::logs::TransformedLogData;
use crate::models::datasets::traces::TransformedTraceData;
use crate::models::datasets::transactions::TransformedTransactionData;
use crate::storage::bigquery::insert_data;
use crate::utils::strip_html;

const MAX_CHANNEL_CAPACITY: usize = 1024;
const BATCH_SIZE: usize = 10; // Number of blocks to batch together
const MAX_BATCH_WAIT: Duration = Duration::from_secs(5); // Maximum time to wait for a batch

#[derive(Debug)]
pub enum DatasetType {
    Blocks(Vec<TransformedBlockData>),
    Transactions(Vec<TransformedTransactionData>),
    Logs(Vec<TransformedLogData>),
    Traces(Vec<TransformedTraceData>),
}

#[derive(Clone)]
pub struct DataChannels {
    pub blocks_tx: Sender<(Vec<TransformedBlockData>, u64)>,
    pub transactions_tx: Sender<(Vec<TransformedTransactionData>, u64)>,
    pub logs_tx: Sender<(Vec<TransformedLogData>, u64)>,
    pub traces_tx: Sender<(Vec<TransformedTraceData>, u64)>,
    shutdown: broadcast::Sender<()>,
    // Track last processed block for each worker
    last_block_processed: Arc<WorkerProgress>,
}

struct WorkerProgress {
    blocks: AtomicU64,
    transactions: AtomicU64,
    logs: AtomicU64,
    traces: AtomicU64,
}

impl DataChannels {
    pub fn shutdown_signal(&self) -> broadcast::Receiver<()> {
        self.shutdown.subscribe()
    }

    pub async fn shutdown(self, end_block: Option<u64>) -> Result<()> {
        // Signal all workers to shutdown
        if let Err(e) = self.shutdown.send(()) {
            error!("Failed to send shutdown signal to workers: {}", e);
        }

        let timeout = StdDuration::from_secs(60 * 5);
        let start = Instant::now();

        while start.elapsed() < timeout {
            if let Some(target) = end_block {
                // Check if all workers have processed up to the end block
                let all_complete = self.all_workers_completed(target);
                if all_complete {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    info!("All workers completed processing up to block {}", target);
                    return Ok(());
                }
                debug!(
                    "Waiting for workers to complete. Progress: blocks={}, txs={}, logs={}, traces={}", 
                    self.last_block_processed.blocks.load(Ordering::Relaxed),
                    self.last_block_processed.transactions.load(Ordering::Relaxed),
                    self.last_block_processed.logs.load(Ordering::Relaxed),
                    self.last_block_processed.traces.load(Ordering::Relaxed),
                );
            } else if self.all_channels_empty() {
                tokio::time::sleep(Duration::from_secs(1)).await;
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        error!("Shutdown timed out with data still being processed");
        Err(anyhow!("Shutdown timed out"))
    }

    fn all_workers_completed(&self, target_block: u64) -> bool {
        let blocks = self.last_block_processed.blocks.load(Ordering::Relaxed);
        let txs = self
            .last_block_processed
            .transactions
            .load(Ordering::Relaxed);
        let logs = self.last_block_processed.logs.load(Ordering::Relaxed);
        let traces = self.last_block_processed.traces.load(Ordering::Relaxed);

        blocks >= target_block
            && txs >= target_block
            && logs >= target_block
            && traces >= target_block
    }

    // Helper methods to update progress
    pub fn update_blocks_progress(&self, block: u64) {
        self.last_block_processed
            .blocks
            .store(block, Ordering::Relaxed);
    }

    pub fn update_transactions_progress(&self, block: u64) {
        self.last_block_processed
            .transactions
            .store(block, Ordering::Relaxed);
    }

    pub fn update_logs_progress(&self, block: u64) {
        self.last_block_processed
            .logs
            .store(block, Ordering::Relaxed);
    }

    pub fn update_traces_progress(&self, block: u64) {
        self.last_block_processed
            .traces
            .store(block, Ordering::Relaxed);
    }

    fn all_channels_empty(&self) -> bool {
        // Check if all channels have processed their remaining items
        self.blocks_tx.capacity() == MAX_CHANNEL_CAPACITY
            && self.transactions_tx.capacity() == MAX_CHANNEL_CAPACITY
            && self.logs_tx.capacity() == MAX_CHANNEL_CAPACITY
            && self.traces_tx.capacity() == MAX_CHANNEL_CAPACITY
    }

    pub async fn send_dataset(&self, dataset_type: DatasetType, block_number: u64) {
        match dataset_type {
            DatasetType::Blocks(data) => {
                if let Err(e) = self.blocks_tx.send((data, block_number)).await {
                    error!("Failed to send blocks batch to channel: {}", e);
                }
            }
            DatasetType::Transactions(data) => {
                if let Err(e) = self.transactions_tx.send((data, block_number)).await {
                    error!("Failed to send transactions batch to channel: {}", e);
                }
            }
            DatasetType::Logs(data) => {
                if let Err(e) = self.logs_tx.send((data, block_number)).await {
                    error!("Failed to send logs batch to channel: {}", e);
                }
            }
            DatasetType::Traces(data) => {
                if let Err(e) = self.traces_tx.send((data, block_number)).await {
                    error!("Failed to send traces batch to channel: {}", e);
                }
            }
        }
    }
}

pub async fn setup_channels(chain_name: &str, metrics: Option<&Metrics>) -> Result<DataChannels> {
    let (blocks_tx, mut blocks_rx) = mpsc::channel(MAX_CHANNEL_CAPACITY);
    let (transactions_tx, mut transactions_rx) = mpsc::channel(MAX_CHANNEL_CAPACITY);
    let (logs_tx, mut logs_rx) = mpsc::channel(MAX_CHANNEL_CAPACITY);
    let (traces_tx, mut traces_rx) = mpsc::channel(MAX_CHANNEL_CAPACITY);
    let (shutdown_tx, _) = broadcast::channel(1);

    let progress = Arc::new(WorkerProgress {
        blocks: AtomicU64::new(0),
        transactions: AtomicU64::new(0),
        logs: AtomicU64::new(0),
        traces: AtomicU64::new(0),
    });

    let channels = DataChannels {
        blocks_tx,
        transactions_tx,
        logs_tx,
        traces_tx,
        shutdown: shutdown_tx.clone(),
        last_block_processed: progress.clone(),
    };

    // Clone metrics once at the beginning if it exists
    let metrics = metrics.map(|m| m.clone());

    // Spawn worker for blocks
    let blocks_dataset = chain_name.to_owned();
    let mut shutdown_rx = shutdown_tx.subscribe();
    let channels_clone = channels.clone();
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        let result = async {
            let mut batch = Vec::new();
            let mut min_block = u64::MAX;
            let mut max_block = 0;
            let mut block_count = 0;
            let mut block_numbers = std::collections::HashSet::new();
            let mut last_batch_time = Instant::now();

            loop {
                tokio::select! {
                    Some((blocks, block_number)) = blocks_rx.recv() => {
                        batch.extend(blocks);
                        min_block = min_block.min(block_number);
                        max_block = max_block.max(block_number);

                        // Only count each block number once
                        if block_numbers.insert(block_number) {
                            block_count += 1;
                        }

                        // Insert batch if we've collected BATCH_SIZE different blocks or if we've waited too long
                        if block_count >= BATCH_SIZE || last_batch_time.elapsed() >= MAX_BATCH_WAIT {
                            if !batch.is_empty() {
                                if let Err(e) = insert_data(&blocks_dataset, "blocks", batch, (min_block, max_block), metrics_clone.as_ref()).await {
                                    error!("Failed to insert block data: {}", strip_html(&e.to_string()));
                                }
                                channels_clone.update_blocks_progress(max_block);
                                batch = Vec::new();
                                min_block = u64::MAX;
                                max_block = 0;
                                block_count = 0;
                                block_numbers.clear();
                                last_batch_time = Instant::now();
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Blocks worker processing remaining items...");
                        // Process any remaining items in the batch
                        if !batch.is_empty() {
                            if let Err(e) = insert_data(&blocks_dataset, "blocks", batch, (min_block, max_block), metrics_clone.as_ref()).await {
                                error!("Failed to insert final block data: {}", strip_html(&e.to_string()));
                            }
                            channels_clone.update_blocks_progress(max_block);
                        }
                        // Process any remaining items in the channel
                        while let Some((blocks, block_number)) = blocks_rx.recv().await {
                            if let Err(e) = insert_data(&blocks_dataset, "blocks", blocks, (block_number, block_number), metrics_clone.as_ref()).await {
                                error!("Failed to insert final block data: {}", strip_html(&e.to_string()));
                            }
                            channels_clone.update_blocks_progress(block_number);
                        }
                        debug!("Blocks worker completed");
                        break;
                    }
                }
            }
            Ok::<_, anyhow::Error>(())
        }.await;

        if let Err(e) = result {
            error!("Blocks worker error: {}", strip_html(&e.to_string()));
        }
        info!("Blocks worker shut down");
    });

    // Spawn worker for transactions
    let transactions_dataset = chain_name.to_owned();
    let mut shutdown_rx = shutdown_tx.subscribe();
    let channels_clone = channels.clone();
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        let result = async {
            let mut batch = Vec::new();
            let mut min_block = u64::MAX;
            let mut max_block = 0;
            let mut block_count = 0;
            let mut block_numbers = std::collections::HashSet::new();
            let mut last_batch_time = Instant::now();

            loop {
                tokio::select! {
                    Some((transactions, block_number)) = transactions_rx.recv() => {
                        batch.extend(transactions);
                        min_block = min_block.min(block_number);
                        max_block = max_block.max(block_number);

                        // Only count each block number once
                        if block_numbers.insert(block_number) {
                            block_count += 1;
                        }

                        // Insert batch if we've collected BATCH_SIZE different blocks or if we've waited too long
                        if block_count >= BATCH_SIZE || last_batch_time.elapsed() >= MAX_BATCH_WAIT {
                            if !batch.is_empty() {
                                if let Err(e) = insert_data(&transactions_dataset, "transactions", batch, (min_block, max_block), metrics_clone.as_ref()).await {
                                    error!("Failed to insert transaction data: {}", strip_html(&e.to_string()));
                                }
                                channels_clone.update_transactions_progress(max_block);
                                batch = Vec::new();
                                min_block = u64::MAX;
                                max_block = 0;
                                block_count = 0;
                                block_numbers.clear();
                                last_batch_time = Instant::now();
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Transactions worker processing remaining items...");
                        // Process any remaining items in the batch
                        if !batch.is_empty() {
                            if let Err(e) = insert_data(&transactions_dataset, "transactions", batch, (min_block, max_block), metrics_clone.as_ref()).await {
                                error!("Failed to insert final transaction data: {}", strip_html(&e.to_string()));
                            }
                            channels_clone.update_transactions_progress(max_block);
                        }
                        // Process any remaining items in the channel
                        while let Some((transactions, block_number)) = transactions_rx.recv().await {
                            if let Err(e) = insert_data(&transactions_dataset, "transactions", transactions, (block_number, block_number), metrics_clone.as_ref()).await {
                                error!("Failed to insert final transaction data: {}", strip_html(&e.to_string()));
                            }
                            channels_clone.update_transactions_progress(block_number);
                        }
                        debug!("Transactions worker completed");
                        break;
                    }
                }
            }
            Ok::<_, anyhow::Error>(())
        }.await;

        if let Err(e) = result {
            error!("Transactions worker error: {}", strip_html(&e.to_string()));
        }
        info!("Transactions worker shut down");
    });

    // Spawn worker for logs
    let logs_dataset = chain_name.to_owned();
    let mut shutdown_rx = shutdown_tx.subscribe();
    let channels_clone = channels.clone();
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        let result = async {
            let mut batch = Vec::new();
            let mut min_block = u64::MAX;
            let mut max_block = 0;
            let mut block_count = 0;
            let mut block_numbers = std::collections::HashSet::new();
            let mut last_batch_time = Instant::now();

            loop {
                tokio::select! {
                    Some((logs, block_number)) = logs_rx.recv() => {
                        batch.extend(logs);
                        min_block = min_block.min(block_number);
                        max_block = max_block.max(block_number);

                        // Only count each block number once
                        if block_numbers.insert(block_number) {
                            block_count += 1;
                        }

                        // Insert batch if we've collected BATCH_SIZE different blocks or if we've waited too long
                        if block_count >= BATCH_SIZE || last_batch_time.elapsed() >= MAX_BATCH_WAIT {
                            if !batch.is_empty() {
                                if let Err(e) = insert_data(&logs_dataset, "logs", batch, (min_block, max_block), metrics_clone.as_ref()).await {
                                    error!("Failed to insert log data: {}", strip_html(&e.to_string()));
                                }
                                channels_clone.update_logs_progress(max_block);
                                batch = Vec::new();
                                min_block = u64::MAX;
                                max_block = 0;
                                block_count = 0;
                                block_numbers.clear();
                                last_batch_time = Instant::now();
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Logs worker processing remaining items...");
                        // Process any remaining items in the batch
                        if !batch.is_empty() {
                            if let Err(e) = insert_data(&logs_dataset, "logs", batch, (min_block, max_block), metrics_clone.as_ref()).await {
                                error!("Failed to insert final log data: {}", strip_html(&e.to_string()));
                            }
                            channels_clone.update_logs_progress(max_block);
                        }
                        // Process any remaining items in the channel
                        while let Some((logs, block_number)) = logs_rx.recv().await {
                            if let Err(e) = insert_data(&logs_dataset, "logs", logs, (block_number, block_number), metrics_clone.as_ref()).await {
                                error!("Failed to insert final log data: {}", strip_html(&e.to_string()));
                            }
                            channels_clone.update_logs_progress(block_number);
                        }
                        debug!("Logs worker completed");
                        break;
                    }
                }
            }
            Ok::<_, anyhow::Error>(())
        }.await;

        if let Err(e) = result {
            error!("Logs worker error: {}", strip_html(&e.to_string()));
        }
        info!("Logs worker shut down");
    });

    // Spawn worker for traces
    let traces_dataset = chain_name.to_owned();
    let mut shutdown_rx = shutdown_tx.subscribe();
    let channels_clone = channels.clone();
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        let result = async {
            let mut batch = Vec::new();
            let mut min_block = u64::MAX;
            let mut max_block = 0;
            let mut block_count = 0;
            let mut block_numbers = std::collections::HashSet::new();
            let mut last_batch_time = Instant::now();

            loop {
                tokio::select! {
                    Some((traces, block_number)) = traces_rx.recv() => {
                        batch.extend(traces);
                        min_block = min_block.min(block_number);
                        max_block = max_block.max(block_number);

                        // Only count each block number once
                        if block_numbers.insert(block_number) {
                            block_count += 1;
                        }

                        // Insert batch if we've collected BATCH_SIZE different blocks or if we've waited too long
                        if block_count >= BATCH_SIZE || last_batch_time.elapsed() >= MAX_BATCH_WAIT {
                            if !batch.is_empty() {
                                if let Err(e) = insert_data(&traces_dataset, "traces", batch, (min_block, max_block), metrics_clone.as_ref()).await {
                                    error!("Failed to insert trace data: {}", strip_html(&e.to_string()));
                                }
                                channels_clone.update_traces_progress(max_block);
                                batch = Vec::new();
                                min_block = u64::MAX;
                                max_block = 0;
                                block_count = 0;
                                block_numbers.clear();
                                last_batch_time = Instant::now();
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Traces worker processing remaining items...");
                        // Process any remaining items in the batch
                        if !batch.is_empty() {
                            if let Err(e) = insert_data(&traces_dataset, "traces", batch, (min_block, max_block), metrics_clone.as_ref()).await {
                                error!("Failed to insert final trace data: {}", strip_html(&e.to_string()));
                            }
                            channels_clone.update_traces_progress(max_block);
                        }
                        // Process any remaining items in the channel
                        while let Some((traces, block_number)) = traces_rx.recv().await {
                            if let Err(e) = insert_data(&traces_dataset, "traces", traces, (block_number, block_number), metrics_clone.as_ref()).await {
                                error!("Failed to insert final trace data: {}", strip_html(&e.to_string()));
                            }
                            channels_clone.update_traces_progress(block_number);
                        }
                        debug!("Traces worker completed");
                        break;
                    }
                }
            }
            Ok::<_, anyhow::Error>(())
        }.await;

        if let Err(e) = result {
            error!("Traces worker error: {}", strip_html(&e.to_string()));
        }
        info!("Traces worker shut down");
    });

    Ok(channels)
}

// Initialize BigQuery dataset and tables
pub async fn initialize_storage(
    chain_name: &str,
    dataset_location: &str,
    datasets: &[String],
    chain: Chain,
) -> Result<()> {
    info!("Initializing storage for chain: {}", chain_name);

    // Create dataset if it doesn't exist
    bigquery::create_dataset(chain_name, dataset_location).await?;

    // Create tables if they don't exist
    for table in ["blocks", "logs", "transactions", "traces"] {
        if datasets.contains(&table.to_owned()) {
            bigquery::create_table(chain_name, table, chain).await?;
        }
    }

    info!("Storage initialized successfully");
    Ok(())
}
