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
use crate::utils::retry::{retry, RetryConfig};

pub async fn get_chain_id<T, N>(
    provider: &dyn Provider<T, N>,
    metrics: Option<&Metrics>,
) -> Result<u64>
where
    T: Transport + Clone,
    N: Network,
{
    let retry_config = RetryConfig::default();

    retry(
        || async {
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

            result.map_err(|e| anyhow!("RPC error: {}", e))
        },
        &retry_config,
        "get_chain_id",
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
    retry(
        || async {
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
                .map_err(|e| anyhow!("RPC error: {}", e))
                .map(BlockNumberOrTag::Number)
        },
        &retry_config,
        "get_latest_block_number",
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
    retry(
        || async {
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

            result.map_err(|e| anyhow!("RPC error: {}", e))
        },
        &retry_config,
        &format!(
            "get_block_by_number({})",
            block_number.as_number().unwrap_or_default()
        ),
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
    retry(
        || async {
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

            result.map_err(|e| anyhow!("RPC error: {}", e))
        },
        &retry_config,
        &match block {
            BlockId::Number(num) => format!(
                "get_block_receipts({})",
                num.as_number().unwrap_or_default()
            ),
            BlockId::Hash(hash) => format!("get_block_receipts({})", hash),
        },
    )
    .await
}

// pub async fn debug_trace_block_by_number<T, N>(
//     provider: &impl DebugApi<N, T>,
//     block_number: BlockNumberOrTag,
//     trace_options: GethDebugTracingOptions,
//     metrics: Option<&Metrics>,
// ) -> Result<Option<Vec<TraceResult<GethTrace, String>>>>
// where
//     T: Transport + Clone,
//     N: Network,
// {
//     let retry_config = RetryConfig::default();
//     retry(
//         || async {
//             let start = std::time::Instant::now();

//             if let Some(metrics) = metrics {
//                 metrics.rpc_requests.add(
//                     1,
//                     &[
//                         KeyValue::new("chain", metrics.chain_name.clone()),
//                         KeyValue::new("method", "debug_trace_block_by_number"),
//                     ],
//                 );
//             }

//             let result = provider
//                 .debug_trace_block_by_number(block_number, trace_options.clone())
//                 .await;

//             // Record metrics if enabled
//             if let Some(metrics) = metrics {
//                 metrics.rpc_latency.record(
//                     start.elapsed().as_secs_f64(),
//                     &[
//                         KeyValue::new("chain", metrics.chain_name.clone()),
//                         KeyValue::new("method", "debug_trace_block_by_number"),
//                     ],
//                 );
//                 if result.is_err() {
//                     metrics.rpc_errors.add(
//                         1,
//                         &[
//                             KeyValue::new("chain", metrics.chain_name.clone()),
//                             KeyValue::new("method", "debug_trace_block_by_number"),
//                         ],
//                     );
//                 }
//             }

//             result.map_err(|e| anyhow!("RPC error: {}", e))
//         },
//         &retry_config,
//         &format!(
//             "debug_trace_block_by_number({})",
//             block_number.as_number().unwrap_or_default()
//         ),
//     )
//     .await
//     .map(Some)
// }

pub async fn debug_trace_transaction_by_hash<T, N>(
    provider: &impl DebugApi<N, T>,
    transaction_hashes: Vec<FixedBytes<32>>,
    trace_options: GethDebugTracingOptions,
    metrics: Option<&Metrics>,
) -> Result<Option<Vec<TraceResult<GethTrace, String>>>>
where
    T: Transport + Clone,
    N: Network,
{
    let retry_config = RetryConfig::default();
    retry(
        || async {
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

            // Collect all transaction traces
            let mut traces = Vec::new();
            for tx_hash in &transaction_hashes {
                let result = provider
                    .debug_trace_transaction(*tx_hash, trace_options.clone())
                    .await;

                match result {
                    Ok(trace) => {
                        traces.push(TraceResult::Success {
                            result: trace,
                            tx_hash: Some(*tx_hash),
                        });
                    }
                    Err(e) => {
                        if e.to_string().contains("-32008") {
                            warn!(
                                "Skipping oversized trace for transaction {}: {}",
                                tx_hash, e
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
                        return Err(anyhow!("RPC error tracing transaction {}: {}", tx_hash, e));
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

            Ok(traces)
        },
        &retry_config,
        "debug_trace_transactions",
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
    let (header, transactions) = if let Some(block) = block {
        (
            block.clone().parse_header(chain)?,
            block.clone().parse_transactions(chain)?,
        )
    } else {
        (vec![], vec![])
    };

    // Parse receipt data if available
    let (transaction_receipts, logs) = if let Some(receipts) = receipts {
        (
            receipts.clone().parse_transaction_receipts(chain)?,
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
    // Build set of common fields I need to pass across datasets (e.g. block_number -> block_time, block_date)
    // Hashmap is likely overkill for now with processing only a single block but will be useful for processing multiple blocks
    let block_map: HashMap<_, _> = parsed_data
        .header
        .clone()
        .into_iter()
        .map(|header| match header {
            RpcHeaderData::Ethereum(eth_header) => (
                eth_header.common.block_number,
                (eth_header.common.block_time, eth_header.common.block_date),
            ),
            RpcHeaderData::ZKsync(zk_header) => (
                zk_header.common.block_number,
                (zk_header.common.block_time, zk_header.common.block_date),
            ),
        })
        .collect();

    // Only transform data for active datasets, otherwise return empty Vec
    let blocks = if active_datasets.contains(&"blocks".to_string()) {
        parsed_data.clone().transform_blocks(chain)?
    } else {
        vec![]
    };

    let transactions = if active_datasets.contains(&"transactions".to_string())
        && !parsed_data.transactions.is_empty()
    {
        parsed_data
            .clone()
            .transform_transactions(chain, block_map.clone())?
    } else {
        vec![]
    };

    let logs = if active_datasets.contains(&"logs".to_string()) && !parsed_data.logs.is_empty() {
        parsed_data.clone().transform_logs(chain)? // Don't need to pass block_map here as logs already have desired fields
    } else {
        vec![]
    };

    let traces =
        if active_datasets.contains(&"traces".to_string()) && !parsed_data.traces.is_empty() {
            parsed_data.clone().transform_traces(chain, block_map)?
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
