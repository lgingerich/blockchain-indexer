pub mod rpc;
pub mod transformations;

use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_network::{
    primitives::BlockTransactionsKind, AnyRpcBlock, AnyTransactionReceipt, Network,
};
use alloy_provider::{ext::DebugApi, Provider};
use alloy_rpc_types_trace::{
    common::TraceResult,
    geth::{GethDebugTracingOptions, GethTrace},
};

use alloy_primitives::FixedBytes;
use alloy_transport::Transport;
use anyhow::{anyhow, Result};
use opentelemetry::KeyValue;
use std::collections::HashMap;
use tracing::warn;

use crate::indexer::rpc::{blocks::BlockParser, receipts::ReceiptParser, traces::TraceParser};
use crate::indexer::transformations::{
    blocks::BlockTransformer, logs::LogTransformer, traces::TraceTransformer,
    transactions::TransactionTransformer,
};
use crate::metrics::Metrics;
use crate::models::common::{Chain, ParsedData, TransformedData};
use crate::models::datasets::blocks::RpcHeaderData;
use crate::models::datasets::logs::RpcLogReceiptData;
use crate::models::datasets::traces::RpcTraceData;
use crate::models::datasets::transactions::RpcTransactionData;
// use crate::utils::retry::get_retry_config;
use crate::utils::retry::{RetryConfig, retry};

use alloy_consensus::TxEnvelope;
use alloy_network::AnyTxEnvelope;
use alloy_rpc_types_trace::geth::{
    GethDebugBuiltInTracerType, GethDebugTracerConfig, GethDebugTracerType,
    GethDefaultTracingOptions,
};

pub trait ProviderDebugApi<T, N>: Provider<T, N> + DebugApi<N, T>
where
    T: Transport + Clone + Send + Sync,
    N: Network,
{
}

impl<T, N, U> ProviderDebugApi<T, N> for U
where
    U: Provider<T, N> + DebugApi<N, T>,
    T: Transport + Clone + Send + Sync,
    N: Network,
{
}

pub async fn get_chain_id<T, N>(
    provider: &dyn Provider<T, N>,
    metrics: Option<&Metrics>,
) -> Result<u64>
where
    T: Transport + Clone,
    N: Network,
{
    let retry_config = RetryConfig::default();
    retry(|| async {
        let start = std::time::Instant::now();

        // Record metrics if enabled
        if let Some(metrics) = metrics {
            metrics.rpc_requests.add(
                1,
                &[
                    KeyValue::new("chain", metrics.chain_name.clone()),
                    KeyValue::new("method", "get_chain_id"),
                ],
            );
        }

        let result = provider.get_chain_id().await;

        // Record metrics if enabled
        if let Some(metrics) = metrics {
            metrics.rpc_latency.record(
                start.elapsed().as_secs_f64(),
                &[
                    KeyValue::new("chain", metrics.chain_name.clone()),
                    KeyValue::new("method", "get_chain_id"),
                ],
            );

            if result.is_err() {
                metrics.rpc_errors.add(
                    1,
                    &[
                        KeyValue::new("chain", metrics.chain_name.clone()),
                        KeyValue::new("method", "get_chain_id"),
                    ],
                );
            }
        }

        result.map_err(|e| {
            warn!("Failed to get chain ID. Error details:\n{:#?}", e);
            anyhow!("RPC error: {}", e)
        })
    }, 
    &retry_config,
    "get_chain_id"
    )
    .await
}

pub async fn get_latest_block_number<T, N>(
    provider: &dyn Provider<T, N>,
    metrics: Option<&Metrics>,
) -> Result<BlockNumberOrTag>
where
    T: Transport + Clone,
    N: Network,
{
    let retry_config = RetryConfig::default();
    retry(|| async {
        let start = std::time::Instant::now();

        if let Some(metrics) = metrics {
            metrics.rpc_requests.add(
                1,
                &[
                    KeyValue::new("chain", metrics.chain_name.clone()),
                    KeyValue::new("method", "get_latest_block_number"),
                ],
            );
        }

        let result = provider.get_block_number().await;

        // Record metrics if enabled
        if let Some(metrics) = metrics {
            metrics.rpc_latency.record(
                start.elapsed().as_secs_f64(),
                &[
                    KeyValue::new("chain", metrics.chain_name.clone()),
                    KeyValue::new("method", "get_latest_block_number"),
                ],
            );
            if result.is_err() {
                metrics.rpc_errors.add(
                    1,
                    &[
                        KeyValue::new("chain", metrics.chain_name.clone()),
                        KeyValue::new("method", "get_latest_block_number"),
                    ],
                );
            }
        }

        result
            .map_err(|e| {
                warn!("Failed to get latest block number. Error details:\n{:#?}", e);
                anyhow!("RPC error: {}", e)
            })
            .map(BlockNumberOrTag::Number)
    }, 
    &retry_config,
    "get_latest_block_number"
    )
    .await
}

pub async fn get_block_by_number<T, N>(
    provider: &dyn Provider<T, N>,
    block_number: BlockNumberOrTag,
    kind: BlockTransactionsKind,
    metrics: Option<&Metrics>,
) -> Result<Option<N::BlockResponse>>
where
    T: Transport + Clone,
    N: Network,
{
    let retry_config = RetryConfig::default();
    retry(|| async {
        let start = std::time::Instant::now();

        if let Some(metrics) = metrics {
            metrics.rpc_requests.add(
                1,
                &[
                    KeyValue::new("chain", metrics.chain_name.clone()),
                    KeyValue::new("method", "get_block_by_number"),
                ],
            );
        }

        let result = provider.get_block_by_number(block_number, kind).await;

        // Record metrics if enabled
        if let Some(metrics) = metrics {
            metrics.rpc_latency.record(
                start.elapsed().as_secs_f64(),
                &[
                    KeyValue::new("chain", metrics.chain_name.clone()),
                    KeyValue::new("method", "get_block_by_number"),
                ],
            );
            if result.is_err() {
                metrics.rpc_errors.add(
                    1,
                    &[
                        KeyValue::new("chain", metrics.chain_name.clone()),
                        KeyValue::new("method", "get_block_by_number"),
                    ],
                );
            }
        }

        result.map_err(|e| {
            warn!("Failed to get block by number {}. Error details:\n{:#?}", block_number, e);
            anyhow!("RPC error: {}", e)
        })
    }, 
    &retry_config,
    "get_block_by_number"
    )
    .await
}

pub async fn get_block_receipts<T, N>(
    provider: &dyn Provider<T, N>,
    block: BlockId,
    metrics: Option<&Metrics>,
) -> Result<Option<Vec<N::ReceiptResponse>>>
where
    T: Transport + Clone,
    N: Network,
{
    let retry_config = RetryConfig::default();
    retry(|| async {
        let start = std::time::Instant::now();

        if let Some(metrics) = metrics {
            metrics.rpc_requests.add(
                1,
                &[
                    KeyValue::new("chain", metrics.chain_name.clone()),
                    KeyValue::new("method", "get_block_receipts"),
                ],
            );
        }

        let result = provider.get_block_receipts(block).await;

        // Record metrics if enabled
        if let Some(metrics) = metrics {
            metrics.rpc_latency.record(
                start.elapsed().as_secs_f64(),
                &[
                    KeyValue::new("chain", metrics.chain_name.clone()),
                    KeyValue::new("method", "get_block_receipts"),
                ],
            );
            if result.is_err() {
                metrics.rpc_errors.add(
                    1,
                    &[
                        KeyValue::new("chain", metrics.chain_name.clone()),
                        KeyValue::new("method", "get_block_receipts"),
                    ],
                );
            }
        }

        result.map_err(|e| {
            warn!("Failed to get block receipts for block {}. Error details:\n{:#?}", block, e);
            anyhow!("RPC error: {}", e)
        })
    }, 
    &retry_config,
    "get_block_receipts"
    )
    .await
}

pub async fn debug_trace_transaction_by_hash<T, N>(
    provider: &(impl DebugApi<N, T> + ?Sized),
    transaction_hashes: Vec<FixedBytes<32>>,
    trace_options: GethDebugTracingOptions,
    metrics: Option<&Metrics>,
) -> Result<Option<Vec<TraceResult<GethTrace, String>>>>
where
    T: Transport + Clone,
    N: Network,
{
    const BATCH_SIZE: usize = 10; // Configurable batch size

    // Retry::spawn(get_retry_config("debug_trace_transaction"), || async {
    let retry_config = RetryConfig::default();
    retry(|| async {
        let start = std::time::Instant::now();

        if let Some(metrics) = metrics {
            metrics.rpc_requests.add(
                1,
                &[
                    KeyValue::new("chain", metrics.chain_name.clone()),
                    KeyValue::new("method", "debug_trace_transaction"),
                ],
            );
        }

        // Process transactions in batches
        let mut all_traces = Vec::with_capacity(transaction_hashes.len());
        for tx_batch in transaction_hashes.chunks(BATCH_SIZE) {
            let mut futures = Vec::with_capacity(tx_batch.len());

            // Create futures for each transaction in the batch
            for tx_hash in tx_batch {
                futures.push(provider.debug_trace_transaction(*tx_hash, trace_options.clone()));
            }

            // Execute batch of futures concurrently
            let batch_results = futures::future::join_all(futures).await;

            // Process results from the batch
            for (idx, result) in batch_results.into_iter().enumerate() {
                match result {
                    Ok(trace) => {
                        all_traces.push(TraceResult::Success {
                            result: trace,
                            tx_hash: Some(tx_batch[idx]),
                        });
                    }
                    Err(e) => {
                        if e.to_string().contains("-32008") {
                            warn!(
                                "Skipping oversized trace for transaction {}: {}",
                                tx_batch[idx], e
                            );
                            continue;
                        }

                        if let Some(metrics) = metrics {
                            metrics.rpc_errors.add(
                                1,
                                &[
                                    KeyValue::new("chain", metrics.chain_name.clone()),
                                    KeyValue::new("method", "debug_trace_transaction"),
                                ],
                            );
                        }
                        warn!(
                            "Failed to trace transaction {} with error: {}",
                            tx_batch[idx],
                            format!("{:?}", e)
                        );
                        return Err(anyhow!(
                            "RPC error tracing transaction {}: {}",
                            tx_batch[idx],
                            e
                        ));
                    }
                }
            }
        }

        // Record metrics if enabled
        if let Some(metrics) = metrics {
            metrics.rpc_latency.record(
                start.elapsed().as_secs_f64(),
                &[
                    KeyValue::new("chain", metrics.chain_name.clone()),
                    KeyValue::new("method", "debug_trace_transaction"),
                ],
            );
        }

        Ok(all_traces)
    }, 
    &retry_config,
    "debug_trace_transaction"
    )
    .await
    .map(Some)
}

pub async fn parse_data(
    chain: Chain,
    chain_id: u64,
    block_number: u64,
    block: Option<AnyRpcBlock>,
    receipts: Option<Vec<AnyTransactionReceipt>>,
    traces: Option<Vec<TraceResult<GethTrace, String>>>,
) -> Result<ParsedData> {
    // Parse block data if available
    let (header, transactions) = if let Some(block) = &block {
        (block.parse_header(chain)?, block.parse_transactions(chain)?)
    } else {
        (vec![], vec![])
    };

    // Parse receipt data if available
    let (transaction_receipts, logs) = if let Some(receipts) = &receipts {
        (
            receipts.parse_transaction_receipts(chain)?,
            receipts.parse_log_receipts(chain)?,
        )
    } else {
        (vec![], vec![])
    };

    // Parse traces if available
    let traces = if let Some(traces) = traces {
        traces.parse_traces(chain, block_number)?
    } else {
        vec![]
    };

    Ok(ParsedData {
        chain_id,
        header,
        transactions,
        transaction_receipts,
        logs,
        traces,
    })
}

pub async fn transform_data(
    chain: Chain,
    parsed_data: ParsedData,
    active_datasets: &[String],
) -> Result<TransformedData> {
    let ParsedData {
        chain_id,
        header,
        transactions,
        transaction_receipts,
        logs,
        traces,
    } = parsed_data;

    // Build block_map from header data
    let block_map: HashMap<_, _, _> = header
        .iter()
        .map(|header| match header {
            RpcHeaderData::Ethereum(eth_header) => (
                eth_header.common.block_number,
                (
                    eth_header.common.block_time,
                    eth_header.common.block_date,
                    eth_header.common.block_hash,
                ),
            ),
            RpcHeaderData::ZKsync(zk_header) => (
                zk_header.common.block_number,
                (
                    zk_header.common.block_time,
                    zk_header.common.block_date,
                    zk_header.common.block_hash,
                ),
            ),
        })
        .collect();

    // Build transaction index map
    let tx_index_map: HashMap<_, _> = transactions
        .iter()
        .map(|tx| match tx {
            RpcTransactionData::Ethereum(t) => (t.common.tx_hash, t.common.tx_index),
            RpcTransactionData::ZKsync(t) => (t.common.tx_hash, t.common.tx_index),
        })
        .collect();

    let blocks = if active_datasets.contains(&"blocks".to_string()) {
        <RpcHeaderData as BlockTransformer>::transform_blocks(header, chain, chain_id)?
    } else {
        vec![]
    };

    let transactions =
        if active_datasets.contains(&"transactions".to_string()) && !transactions.is_empty() {
            <RpcTransactionData as TransactionTransformer>::transform_transactions(
                transactions,
                transaction_receipts,
                chain,
                chain_id,
                &block_map,
            )?
        } else {
            vec![]
        };

    let logs = if active_datasets.contains(&"logs".to_string()) && !logs.is_empty() {
        <RpcLogReceiptData as LogTransformer>::transform_logs(logs, chain, chain_id)?
    } else {
        vec![]
    };

    let traces = if active_datasets.contains(&"traces".to_string()) && !traces.is_empty() {
        <RpcTraceData as TraceTransformer>::transform_traces(
            traces,
            chain,
            chain_id,
            &block_map,
            &tx_index_map,
        )?
    } else {
        vec![]
    };

    Ok(TransformedData {
        blocks,
        transactions,
        logs,
        traces,
    })
}

pub async fn process_block<T, N>(
    provider: &impl ProviderDebugApi<T, N>,
    block_number: BlockNumberOrTag,
    chain: Chain,
    chain_id: u64,
    datasets: &[String],
    metrics: Option<&Metrics>,
) -> Result<TransformedData>
where
    T: Transport + Clone + Send + Sync,
    N: Network<BlockResponse = AnyRpcBlock, ReceiptResponse = AnyTransactionReceipt>,
{
    // Track which RPC responses we need to fetch
    let need_block =
        datasets.contains(&"blocks".to_string()) || datasets.contains(&"transactions".to_string());
    let need_receipts =
        datasets.contains(&"logs".to_string()) || datasets.contains(&"transactions".to_string());
    let need_traces = datasets.contains(&"traces".to_string());

    // Fetch block data if needed
    let block = if need_block {
        Some(
            get_block_by_number(provider, block_number, BlockTransactionsKind::Full, metrics)
                .await?
                .ok_or_else(|| anyhow!("Provider returned no block"))?,
        )
    } else {
        None
    };

    // Fetch receipts if needed
    let receipts = if need_receipts {
        let block_id = BlockId::Number(block_number);
        Some(
            get_block_receipts(provider, block_id, metrics)
                .await?
                .ok_or_else(|| anyhow!("Provider returned no receipts"))?,
        )
    } else {
        None
    };

    // Fetch traces if needed
    let traces = if need_traces {
        // Get transaction hashes from block if we have it
        let tx_hashes = if let Some(block_data) = &block {
            block_data
                .transactions
                .txns()
                .map(|transaction| match &transaction.inner.inner {
                    AnyTxEnvelope::Ethereum(inner) => match inner {
                        TxEnvelope::Legacy(signed) => {
                            (*signed.hash(), transaction.transaction_index)
                        }
                        TxEnvelope::Eip2930(signed) => {
                            (*signed.hash(), transaction.transaction_index)
                        }
                        TxEnvelope::Eip1559(signed) => {
                            (*signed.hash(), transaction.transaction_index)
                        }
                        TxEnvelope::Eip4844(signed) => {
                            (*signed.hash(), transaction.transaction_index)
                        }
                        TxEnvelope::Eip7702(signed) => {
                            (*signed.hash(), transaction.transaction_index)
                        }
                        _ => (FixedBytes::<32>::ZERO, None),
                    },
                    AnyTxEnvelope::Unknown(unknown) => {
                        (unknown.hash, transaction.transaction_index)
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        let trace_options = GethDebugTracingOptions {
            config: GethDefaultTracingOptions::default(),
            tracer: Some(GethDebugTracerType::BuiltInTracer(
                GethDebugBuiltInTracerType::CallTracer,
            )),
            tracer_config: GethDebugTracerConfig(serde_json::json!({"onlyTopCall": false})),
            timeout: Some("10s".to_string()),
        };

        // Get just the hashes for the trace API
        let hashes: Vec<_> = tx_hashes.into_iter().map(|(hash, _)| hash).collect();

        debug_trace_transaction_by_hash(provider, hashes, trace_options, metrics).await?
    } else {
        None
    };

    // Parse and transform the data
    let parsed_data = parse_data(
        chain,
        chain_id,
        block_number.as_number().unwrap(),
        block,
        receipts,
        traces,
    )
    .await?;

    transform_data(chain, parsed_data, datasets).await
}
