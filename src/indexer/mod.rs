pub mod rpc;
pub mod transformations;

use anyhow::{Context, Result};

use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_network::{AnyRpcBlock, AnyTransactionReceipt, BlockResponse, Network};
use alloy_provider::{ext::DebugApi, Provider};
use alloy_rpc_types_trace::{
    common::TraceResult,
    geth::{GethDebugTracingOptions, GethTrace},
};

use alloy_primitives::FixedBytes;
use std::collections::HashMap;
use tracing::warn;

use crate::indexer::rpc::{blocks::BlockParser, receipts::ReceiptParser, traces::TraceParser};
use crate::indexer::transformations::{
    blocks::BlockTransformer, logs::LogTransformer, traces::TraceTransformer,
    transactions::TransactionTransformer,
};
use crate::metrics::Metrics;
use crate::models::common::{ChainInfo, ParsedData, TransformedData};
use crate::models::datasets::blocks::RpcHeaderData;
use crate::models::datasets::logs::RpcLogReceiptData;
use crate::models::datasets::traces::RpcTraceData;
use crate::models::datasets::transactions::RpcTransactionData;
use crate::utils::retry::{retry, RetryConfig};
use crate::utils::Table;

use alloy_consensus::TxEnvelope;
use alloy_network::AnyTxEnvelope;
use alloy_rpc_types_trace::geth::{
    GethDebugBuiltInTracerType, GethDebugTracerConfig, GethDebugTracerType,
    GethDefaultTracingOptions,
};

pub trait ProviderDebugApi<N>: Provider<N> + DebugApi<N>
where
    N: Network,
{
}

impl<N, U> ProviderDebugApi<N> for U
where
    U: Provider<N> + DebugApi<N>,
    N: Network,
{
}

pub async fn get_chain_id<N>(provider: &dyn Provider<N>, metrics: Option<&Metrics>) -> Result<u64>
where
    N: Network,
{
    let retry_config = RetryConfig::default();
    retry(
        || async {
            let start = std::time::Instant::now();

            // Record metrics if enabled
            if let Some(metrics) = metrics {
                metrics.record_rpc_request("get_chain_id");
            }

            let result = provider
                .get_chain_id()
                .await
                .context("Failed to get chain ID");

            // Record metrics if enabled
            if let Some(metrics) = metrics {
                metrics.record_rpc_latency("get_chain_id", start.elapsed().as_secs_f64());

                if result.is_err() {
                    metrics.record_rpc_error("get_chain_id");
                }
            }

            result
        },
        &retry_config,
        "get_chain_id",
    )
    .await
}

pub async fn get_latest_block_number<N>(
    provider: &dyn Provider<N>,
    metrics: Option<&Metrics>,
) -> Result<BlockNumberOrTag>
where
    N: Network,
{
    let retry_config = RetryConfig::default();
    retry(
        || async {
            let start = std::time::Instant::now();

            if let Some(metrics) = metrics {
                metrics.record_rpc_request("get_latest_block_number");
            }

            let result = provider
                .get_block_number()
                .await
                .context("Failed to get latest block number");

            // Record metrics if enabled
            if let Some(metrics) = metrics {
                metrics
                    .record_rpc_latency("get_latest_block_number", start.elapsed().as_secs_f64());
                if result.is_err() {
                    metrics.record_rpc_error("get_latest_block_number");
                }
            }

            result.map(BlockNumberOrTag::Number)
        },
        &retry_config,
        "get_latest_block_number",
    )
    .await
}

pub async fn get_block_by_number<N>(
    provider: &dyn Provider<N>,
    block_number: BlockNumberOrTag,
    metrics: Option<&Metrics>,
) -> Result<Option<N::BlockResponse>>
where
    N: Network,
{
    let retry_config = RetryConfig::default();
    retry(
        || async {
            let start = std::time::Instant::now();

            if let Some(metrics) = metrics {
                metrics.record_rpc_request("get_block_by_number");
            }

            let result = provider
                .get_block(block_number.into())
                .full()
                .await
                .with_context(|| {
                    format!(
                        "Failed request to get_block() for block number {}",
                        block_number
                    )
                });

            // Record metrics if enabled
            if let Some(metrics) = metrics {
                metrics.record_rpc_latency("get_block_by_number", start.elapsed().as_secs_f64());
                if result.is_err() {
                    metrics.record_rpc_error("get_block_by_number");
                }
            }

            result
        },
        &retry_config,
        "get_block_by_number",
    )
    .await
}

pub async fn get_block_receipts<N>(
    provider: &dyn Provider<N>,
    block: BlockId,
    metrics: Option<&Metrics>,
) -> Result<Option<Vec<N::ReceiptResponse>>>
where
    N: Network,
{
    let retry_config = RetryConfig::default();
    retry(
        || async {
            let start = std::time::Instant::now();

            if let Some(metrics) = metrics {
                metrics.record_rpc_request("get_block_receipts");
            }

            let result = provider.get_block_receipts(block).await.with_context(|| {
                format!("Failed request to get_block_receipts() for block {}", block)
            });

            // Record metrics if enabled
            if let Some(metrics) = metrics {
                metrics.record_rpc_latency("get_block_receipts", start.elapsed().as_secs_f64());
                if result.is_err() {
                    metrics.record_rpc_error("get_block_receipts");
                }
            }

            result
        },
        &retry_config,
        "get_block_receipts",
    )
    .await
}

pub async fn debug_trace_transaction_by_hash<N>(
    provider: &(impl DebugApi<N> + ?Sized),
    transaction_hashes: Vec<FixedBytes<32>>,
    trace_options: GethDebugTracingOptions,
    metrics: Option<&Metrics>,
) -> Result<Option<Vec<TraceResult<GethTrace, String>>>>
where
    N: Network,
{
    const BATCH_SIZE: usize = 10; // Configurable batch size

    let retry_config = RetryConfig::default();
    retry(
        || async {
            let start = std::time::Instant::now();

            if let Some(metrics) = metrics {
                metrics.record_rpc_request("debug_trace_transaction");
            }

            // Process transactions in batches
            let mut all_traces = Vec::with_capacity(transaction_hashes.len());
            for tx_batch in transaction_hashes.chunks(BATCH_SIZE) {
                let mut futures = Vec::with_capacity(tx_batch.len());

                // Create futures for each transaction in the batch
                for tx_hash in tx_batch {
                    let trace_options_clone = trace_options.clone(); // Clone for this specific future
                    futures.push(async move {
                        provider
                            .debug_trace_transaction(*tx_hash, trace_options_clone) // Use the clone
                            .await
                            .with_context(|| {
                                format!(
                                    "Failed request to debug_trace_transaction() for transaction hash {}",
                                    tx_hash
                                )
                            })
                    });
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
                                    tx_batch[idx],
                                    &e.to_string()
                                );
                                continue;
                            }

                            if let Some(metrics) = metrics {
                                metrics.record_rpc_error("debug_trace_transaction");
                            }
                            return Err(e);
                        }
                    }
                }
            }

            // Record metrics if enabled
            if let Some(metrics) = metrics {
                metrics.record_rpc_latency("debug_trace_transaction", start.elapsed().as_secs_f64());
            }

            Ok(Some(all_traces))
        },
        &retry_config,
        "debug_trace_transaction",
    )
    .await
}

pub async fn parse_data(
    chain_info: &ChainInfo,
    block_number: u64,
    block: Option<AnyRpcBlock>,
    receipts: Option<Vec<AnyTransactionReceipt>>,
    traces: Option<Vec<TraceResult<GethTrace, String>>>,
) -> Result<ParsedData> {
    // Parse block data if available
    let (header, transactions) = if let Some(block) = &block {
        (block.parse_header(chain_info)?, block.parse_transactions(chain_info)?)
    } else {
        (vec![], vec![])
    };

    // Parse receipt data if available
    let (transaction_receipts, logs) = if let Some(receipts) = &receipts {
        (
            receipts.parse_transaction_receipts(chain_info)?,
            receipts.parse_log_receipts(chain_info)?,
        )
    } else {
        (vec![], vec![])
    };

    // Parse traces if available
    let traces = if let Some(traces) = traces {
        traces.parse_traces(chain_info, block_number)?
    } else {
        vec![]
    };

    Ok(ParsedData {
        header,
        transactions,
        transaction_receipts,
        logs,
        traces,
    })
}

pub async fn transform_data(
    chain_info: &ChainInfo,
    parsed_data: ParsedData,
    active_datasets: &[Table],
) -> Result<TransformedData> {
    let ParsedData {
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

    let blocks = if active_datasets.contains(&Table::Blocks) {
        <RpcHeaderData as BlockTransformer>::transform_blocks(header, chain_info)?
    } else {
        vec![]
    };

    let transactions =
        if active_datasets.contains(&Table::Transactions) && !transactions.is_empty() {
            <RpcTransactionData as TransactionTransformer>::transform_transactions(
                transactions,
                transaction_receipts,
                chain_info,
                &block_map,
            )?
        } else {
            vec![]
        };

    let logs = if active_datasets.contains(&Table::Logs) && !logs.is_empty() {
        <RpcLogReceiptData as LogTransformer>::transform_logs(logs, chain_info)?
    } else {
        vec![]
    };

    let traces = if active_datasets.contains(&Table::Traces) && !traces.is_empty() {
        <RpcTraceData as TraceTransformer>::transform_traces(
            traces,
            chain_info,
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

pub async fn process_block<N>(
    provider: &impl ProviderDebugApi<N>,
    block_number: BlockNumberOrTag,
    chain_info: &ChainInfo,
    datasets: &[Table],
    metrics: Option<&Metrics>,
) -> Result<TransformedData>
where
    N: Network<BlockResponse = AnyRpcBlock, ReceiptResponse = AnyTransactionReceipt>,
{
    // Track which RPC responses we need to fetch
    let need_block =
        datasets.contains(&Table::Blocks) || datasets.contains(&Table::Transactions);
    let need_receipts =
        datasets.contains(&Table::Logs) || datasets.contains(&Table::Transactions);
    let need_traces = datasets.contains(&Table::Traces);

    // Fetch block data if needed
    let block = if need_block {
        get_block_by_number(provider, block_number, metrics).await?
    } else {
        None
    };

    // Fetch receipts if needed
    let receipts = if need_receipts {
        let block_id = BlockId::Number(block_number);
        get_block_receipts(provider, block_id, metrics).await?
    } else {
        None
    };

    // Fetch traces if needed
    let traces = if need_traces {
        // Get transaction hashes from block if we have it
        let tx_hashes = if let Some(block_data) = &block {
            block_data
                .transactions()
                .txns()
                .map(|transaction| match &*transaction.inner.inner {
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
                    },
                    AnyTxEnvelope::Unknown(unknown) => {
                        (unknown.hash, transaction.transaction_index)
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        // // Fetch traces if needed
        // let traces = if need_traces {
        //     // Get transaction hashes from block if we have it
        //     let tx_hashes = if let Some(block_data) = &block {
        //         block_data
        //             .transactions()
        //             .map(|transaction| match &transaction.inner.inner {
        //                 AnyTxEnvelope::Ethereum(inner) => match inner {
        //                     TxEnvelope::Legacy(signed) => {
        //                         (*signed.hash(), transaction.transaction_index)
        //                     }
        //                     TxEnvelope::Eip2930(signed) => {
        //                         (*signed.hash(), transaction.transaction_index)
        //                     }
        //                     TxEnvelope::Eip1559(signed) => {
        //                         (*signed.hash(), transaction.transaction_index)
        //                     }
        //                     TxEnvelope::Eip4844(signed) => {
        //                         (*signed.hash(), transaction.transaction_index)
        //                     }
        //                     TxEnvelope::Eip7702(signed) => {
        //                         (*signed.hash(), transaction.transaction_index)
        //                     }
        //                     _ => (FixedBytes::<32>::ZERO, None),
        //                 },
        //                 AnyTxEnvelope::Unknown(unknown) => {
        //                     (unknown.hash, transaction.transaction_index)
        //                 }
        //             })
        //             .collect()
        //     } else {
        //         Vec::new()
        //     };

        let trace_options = GethDebugTracingOptions {
            config: GethDefaultTracingOptions::default(),
            tracer: Some(GethDebugTracerType::BuiltInTracer(
                GethDebugBuiltInTracerType::CallTracer,
            )),
            tracer_config: GethDebugTracerConfig(serde_json::json!({"onlyTopCall": false})),
            timeout: Some("60s".to_string()),
        };

        // Get just the hashes for the trace API
        let hashes: Vec<_> = tx_hashes.into_iter().map(|(hash, _)| hash).collect();

        debug_trace_transaction_by_hash(provider, hashes, trace_options, metrics).await?
    } else {
        None
    };

    // Parse and transform the data
    let parsed_data = parse_data(
        chain_info,
        block_number
            .as_number()
            .ok_or_else(|| anyhow::anyhow!("Expected block number, got {:?}", block_number))?,
        block,
        receipts,
        traces,
    )
    .await?;

    transform_data(chain_info, parsed_data, datasets).await
}
