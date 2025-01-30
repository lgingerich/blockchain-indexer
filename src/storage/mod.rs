pub mod bigquery;

use anyhow::{anyhow, Result};
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::broadcast;
use tokio::time::{Duration, Instant};
use std::time::Duration as StdDuration;
use tracing::{error, info, debug};
use opentelemetry::KeyValue;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::metrics::Metrics;
use crate::models::datasets::blocks::TransformedBlockData;
use crate::models::datasets::logs::TransformedLogData;
use crate::models::datasets::traces::TransformedTraceData;
use crate::models::datasets::transactions::TransformedTransactionData;
use crate::storage::bigquery::insert_data_with_retry;

const MAX_CHANNEL_CAPACITY: usize = 64;
const CAPACITY_THRESHOLD: f32 = 0.2; // Apply backpressure when current capacity is 20% of max


// TODO: Improve/condense this whole file

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
        
        let timeout = StdDuration::from_secs(30);
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
        let txs = self.last_block_processed.transactions.load(Ordering::Relaxed);
        let logs = self.last_block_processed.logs.load(Ordering::Relaxed);
        let traces = self.last_block_processed.traces.load(Ordering::Relaxed);

        blocks >= target_block && 
        txs >= target_block && 
        logs >= target_block && 
        traces >= target_block
    }

    // Helper methods to update progress
    pub fn update_blocks_progress(&self, block: u64) {
        self.last_block_processed.blocks.store(block, Ordering::Relaxed);
    }

    pub fn update_transactions_progress(&self, block: u64) {
        self.last_block_processed.transactions.store(block, Ordering::Relaxed);
    }

    pub fn update_logs_progress(&self, block: u64) {
        self.last_block_processed.logs.store(block, Ordering::Relaxed);
    }

    pub fn update_traces_progress(&self, block: u64) {
        self.last_block_processed.traces.store(block, Ordering::Relaxed);
    }

    fn all_channels_empty(&self) -> bool {
        // Check if all channels have processed their remaining items
        self.blocks_tx.capacity() == MAX_CHANNEL_CAPACITY &&
        self.transactions_tx.capacity() == MAX_CHANNEL_CAPACITY &&
        self.logs_tx.capacity() == MAX_CHANNEL_CAPACITY &&
        self.traces_tx.capacity() == MAX_CHANNEL_CAPACITY
    }

    pub async fn check_capacity(&self, metrics: Option<&Metrics>) -> Result<bool> {
        // Get current capacities (number of available slots, NOT how many slots are used)
        let blocks_capacity = self.blocks_tx.capacity();
        let transactions_capacity = self.transactions_tx.capacity();
        let logs_capacity = self.logs_tx.capacity();
        let traces_capacity = self.traces_tx.capacity();

        // Record current capacities
        if let Some(metrics) = metrics {
            metrics.channel_capacity.record(
                blocks_capacity as u64,
                &[KeyValue::new("channel", "blocks")],
            );
            metrics.channel_capacity.record(
                transactions_capacity as u64,
                &[KeyValue::new("channel", "transactions")],
            );
            metrics.channel_capacity.record(
                logs_capacity as u64,
                &[KeyValue::new("channel", "logs")],
            );
            metrics.channel_capacity.record(
                traces_capacity as u64,
                &[KeyValue::new("channel", "traces")],
            );
        }

        // Apply backpressure when available capacity is low (meaning channel is getting full)
        // If available capacity is <= 20% of max, then the channel is >= 80% full
        if (blocks_capacity as f32 / MAX_CHANNEL_CAPACITY as f32) <= CAPACITY_THRESHOLD ||
           (transactions_capacity as f32 / MAX_CHANNEL_CAPACITY as f32) <= CAPACITY_THRESHOLD ||
           (logs_capacity as f32 / MAX_CHANNEL_CAPACITY as f32) <= CAPACITY_THRESHOLD ||
           (traces_capacity as f32 / MAX_CHANNEL_CAPACITY as f32) <= CAPACITY_THRESHOLD {
            info!("Channel within {}% of max capacity", (1.0 - CAPACITY_THRESHOLD) * 100.0);
            return Ok(false);
        }

        Ok(true)
    }
}

pub async fn setup_channels(chain_name: &str) -> Result<DataChannels> {
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

    // Spawn worker for blocks
    let blocks_dataset = chain_name.to_owned();
    let mut shutdown_rx = shutdown_tx.subscribe();
    let channels_clone = channels.clone();
    tokio::spawn(async move {
        let result = async {
            loop {
                tokio::select! {
                    Some((blocks, block_number)) = blocks_rx.recv() => {
                        if let Err(e) = insert_data_with_retry(&blocks_dataset, "blocks", blocks, block_number).await {
                            error!("Failed to insert block data: {}", e);
                        }
                        channels_clone.update_blocks_progress(block_number);
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Blocks worker processing remaining items...");
                        while let Some((blocks, block_number)) = blocks_rx.recv().await {
                            if let Err(e) = insert_data_with_retry(&blocks_dataset, "blocks", blocks, block_number).await {
                                error!("Failed to insert final block data: {}", e);
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
            error!("Blocks worker error: {}", e);
        }
        info!("Blocks worker shut down");
    });

    // Spawn worker for transactions
    let transactions_dataset = chain_name.to_owned();
    let mut shutdown_rx = shutdown_tx.subscribe();
    let channels_clone = channels.clone();
    tokio::spawn(async move {
        let result = async {
            loop {
                tokio::select! {
                    Some((transactions, block_number)) = transactions_rx.recv() => {
                        if let Err(e) = insert_data_with_retry(&transactions_dataset, "transactions", transactions, block_number).await {
                            error!("Failed to insert transaction data: {}", e);
                        }
                        channels_clone.update_transactions_progress(block_number);
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Transactions worker processing remaining items...");
                        while let Some((transactions, block_number)) = transactions_rx.recv().await {
                            if let Err(e) = insert_data_with_retry(&transactions_dataset, "transactions", transactions, block_number).await {
                                error!("Failed to insert final transaction data: {}", e);
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
            error!("Transactions worker error: {}", e);
        }
        info!("Transactions worker shut down");
    });

    // Spawn worker for logs
    let logs_dataset = chain_name.to_owned();
    let mut shutdown_rx = shutdown_tx.subscribe();
    let channels_clone = channels.clone();
    tokio::spawn(async move {
        let result = async {
            loop {
                tokio::select! {
                    Some((logs, block_number)) = logs_rx.recv() => {
                        if let Err(e) = insert_data_with_retry(&logs_dataset, "logs", logs, block_number).await {
                            error!("Failed to insert log data: {}", e);
                        }
                        channels_clone.update_logs_progress(block_number);
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Logs worker processing remaining items...");
                        while let Some((logs, block_number)) = logs_rx.recv().await {
                            if let Err(e) = insert_data_with_retry(&logs_dataset, "logs", logs, block_number).await {
                                error!("Failed to insert final log data: {}", e);
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
            error!("Logs worker error: {}", e);
        }
        info!("Logs worker shut down");
    });

    // Spawn worker for traces
    let traces_dataset = chain_name.to_owned();
    let mut shutdown_rx = shutdown_tx.subscribe();
    let channels_clone = channels.clone();
    tokio::spawn(async move {
        let result = async {
            loop {
                tokio::select! {
                    Some((traces, block_number)) = traces_rx.recv() => {
                        if let Err(e) = insert_data_with_retry(&traces_dataset, "traces", traces, block_number).await {
                            error!("Failed to insert trace data: {}", e);
                        }
                        channels_clone.update_traces_progress(block_number);
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Traces worker processing remaining items...");
                        while let Some((traces, block_number)) = traces_rx.recv().await {
                            if let Err(e) = insert_data_with_retry(&traces_dataset, "traces", traces, block_number).await {
                                error!("Failed to insert final trace data: {}", e);
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
            error!("Traces worker error: {}", e);
        }
        info!("Traces worker shut down");
    });

    Ok(channels)
}
