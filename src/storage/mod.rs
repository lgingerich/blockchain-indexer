pub mod bigquery;

use anyhow::{anyhow, Result};
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::broadcast;
use tokio::time::{Duration, Instant};
use std::time::Duration as StdDuration;
use tracing::{error, info, debug};
use opentelemetry::KeyValue;

use crate::metrics::Metrics;
use crate::models::datasets::blocks::TransformedBlockData;
use crate::models::datasets::logs::TransformedLogData;
use crate::models::datasets::traces::TransformedTraceData;
use crate::models::datasets::transactions::TransformedTransactionData;
use crate::storage::bigquery::insert_data_with_retry;

const MAX_CHANNEL_CAPACITY: usize = 64;
const CAPACITY_THRESHOLD: f32 = 0.2; // Apply backpressure when current capacity is 20% of max

#[derive(Clone)]
pub struct DataChannels {
    pub blocks_tx: Sender<(Vec<TransformedBlockData>, u64)>,
    pub transactions_tx: Sender<(Vec<TransformedTransactionData>, u64)>,
    pub logs_tx: Sender<(Vec<TransformedLogData>, u64)>,
    pub traces_tx: Sender<(Vec<TransformedTraceData>, u64)>,
    shutdown: broadcast::Sender<()>,
}

impl DataChannels {
    pub fn shutdown_signal(&self) -> broadcast::Receiver<()> {
        self.shutdown.subscribe()
    }

    pub async fn shutdown(self) -> Result<()> {
        // Signal all workers to shutdown
        if let Err(e) = self.shutdown.send(()) {
            error!("Failed to send shutdown signal to workers: {}", e);
        }
        
        // Wait for channels to clear
        let timeout = StdDuration::from_secs(30);
        let start = Instant::now();
        
        while start.elapsed() < timeout {
            if self.all_channels_empty() {
                info!("All data channels cleared and workers shut down");
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        error!("Shutdown timed out with data still in channels");
        Err(anyhow!("Shutdown timed out"))
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
    let (blocks_tx, mut blocks_rx) = mpsc::channel::<(Vec<TransformedBlockData>, u64)>(MAX_CHANNEL_CAPACITY);
    let (transactions_tx, mut transactions_rx) = mpsc::channel::<(Vec<TransformedTransactionData>, u64)>(MAX_CHANNEL_CAPACITY);
    let (logs_tx, mut logs_rx) = mpsc::channel::<(Vec<TransformedLogData>, u64)>(MAX_CHANNEL_CAPACITY);
    let (traces_tx, mut traces_rx) = mpsc::channel::<(Vec<TransformedTraceData>, u64)>(MAX_CHANNEL_CAPACITY);
    let (shutdown_tx, _) = broadcast::channel(1);

    // Spawn worker for blocks
    let blocks_dataset = chain_name.to_owned();
    let mut shutdown_rx = shutdown_tx.subscribe();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some((blocks, block_number)) = blocks_rx.recv() => {
                    if let Err(e) = insert_data_with_retry(&blocks_dataset, "blocks", blocks, block_number).await {
                        error!("Failed to insert block data: {}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    debug!("Blocks worker processing remaining items...");
                    while let Ok(Some((blocks, block_number))) = tokio::time::timeout(
                        Duration::from_secs(1), 
                        blocks_rx.recv()
                    ).await {
                        if let Err(e) = insert_data_with_retry(&blocks_dataset, "blocks", blocks, block_number).await {
                            error!("Failed to insert final block data: {}", e);
                        }
                    }
                    debug!("Blocks worker completed");
                    break;
                }
            }
        }
        info!("Blocks worker shut down");
    });

    // Spawn worker for transactions
    let transactions_dataset = chain_name.to_owned();
    let mut shutdown_rx = shutdown_tx.subscribe();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some((transactions, block_number)) = transactions_rx.recv() => {
                    if let Err(e) = insert_data_with_retry(&transactions_dataset, "transactions", transactions, block_number).await {
                        error!("Failed to insert transaction data: {}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    debug!("Transactions worker processing remaining items...");
                    while let Ok(Some((transactions, block_number))) = tokio::time::timeout(
                        Duration::from_secs(1), 
                        transactions_rx.recv()
                    ).await {
                        if let Err(e) = insert_data_with_retry(&transactions_dataset, "transactions", transactions, block_number).await {
                            error!("Failed to insert final transaction data: {}", e);
                        }
                    }
                    debug!("Transactions worker completed");
                    break;
                }
            }
        }
        info!("Transactions worker shut down");
    });

    // Spawn worker for logs
    let logs_dataset = chain_name.to_owned();
    let mut shutdown_rx = shutdown_tx.subscribe();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some((logs, block_number)) = logs_rx.recv() => {
                    if let Err(e) = insert_data_with_retry(&logs_dataset, "logs", logs, block_number).await {
                        error!("Failed to insert log data: {}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    debug!("Logs worker processing remaining items...");
                    while let Ok(Some((logs, block_number))) = tokio::time::timeout(
                        Duration::from_secs(1), 
                        logs_rx.recv()
                    ).await {
                        if let Err(e) = insert_data_with_retry(&logs_dataset, "logs", logs, block_number).await {
                            error!("Failed to insert final log data: {}", e);
                        }
                    }
                    debug!("Logs worker completed");
                    break;
                }
            }
        }
        info!("Logs worker shut down");
    });

    // Spawn worker for traces
    let traces_dataset = chain_name.to_owned();
    let mut shutdown_rx = shutdown_tx.subscribe();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some((traces, block_number)) = traces_rx.recv() => {
                    if let Err(e) = insert_data_with_retry(&traces_dataset, "traces", traces, block_number).await {
                        error!("Failed to insert trace data: {}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    debug!("Traces worker processing remaining items...");
                    while let Ok(Some((traces, block_number))) = tokio::time::timeout(
                        Duration::from_secs(1), 
                        traces_rx.recv()
                    ).await {
                        if let Err(e) = insert_data_with_retry(&traces_dataset, "traces", traces, block_number).await {
                            error!("Failed to insert final trace data: {}", e);
                        }
                    }
                    debug!("Traces worker completed");
                    break;
                }
            }
        }
        info!("Traces worker shut down");
    });

    Ok(DataChannels {
        blocks_tx,
        transactions_tx,
        logs_tx,
        traces_tx,
        shutdown: shutdown_tx,
    })
}
