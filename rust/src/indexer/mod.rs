// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

pub mod rpc;
pub mod transformations;

use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_network::{primitives::BlockTransactionsKind, Network};
use alloy_network::{AnyRpcBlock, AnyTransactionReceipt};
use alloy_provider::{ext::DebugApi, Provider, ReqwestProvider};
use alloy_rpc_types_eth::{Block, TransactionReceipt};
use alloy_rpc_types_trace::{
    common::TraceResult,
    geth::{GethDebugTracingOptions, GethTrace},
};
use alloy_transport::{RpcError, Transport};
use anyhow::{anyhow, Result};
use tracing::error;

use crate::indexer::rpc::{blocks::BlockParser, receipts::ReceiptParser, traces::TraceParser};
use crate::indexer::transformations::{
    blocks::BlockTransformer, logs::LogTransformer, traces::TraceTransformer,
    transactions::TransactionTransformer,
};
use crate::models::common::{Chain, ParsedData, TransformedData};
use crate::utils::retry::{retry, RetryConfig};

pub async fn get_chain_id<T, N>(provider: &dyn Provider<T, N>) -> Result<u64>
where
    T: Transport + Clone,
    N: Network,
{
    let retry_config = RetryConfig::default();

    retry(
        || async {
            provider
                .get_chain_id()
                .await
                .map_err(|e| anyhow!("RPC error: {}", e))
        },
        &retry_config,
        "get_chain_id",
    )
    .await
}

pub async fn get_latest_block_number<T, N>(
    provider: &dyn Provider<T, N>,
) -> Result<BlockNumberOrTag>
where
    T: Transport + Clone,
    N: Network,
{
    let retry_config = RetryConfig::default();
    retry(
        || async {
            provider
                .get_block_number()
                .await
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
) -> Result<Option<N::BlockResponse>>
where
    T: Transport + Clone,
    N: Network,
{
    let retry_config = RetryConfig::default();
    retry(
        || async {
            provider
                .get_block_by_number(block_number, kind)
                .await
                .map_err(|e| anyhow!("RPC error: {}", e))
        },
        &retry_config,
        &format!("get_block_by_number({})", block_number),
    )
    .await
}

pub async fn get_block_receipts<T, N>(
    provider: &dyn Provider<T, N>,
    block: BlockId,
) -> Result<Option<Vec<N::ReceiptResponse>>>
where
    T: Transport + Clone,
    N: Network,
{
    let retry_config = RetryConfig::default();
    retry(
        || async {
            provider
                .get_block_receipts(block)
                .await
                .map_err(|e| anyhow!("RPC error: {}", e))
        },
        &retry_config,
        &format!("get_block_receipts({})", block),
    )
    .await
}

pub async fn debug_trace_block_by_number<T, N>(
    provider: &impl DebugApi<N, T>,
    block_number: BlockNumberOrTag,
    trace_options: GethDebugTracingOptions,
) -> Result<Option<Vec<TraceResult<GethTrace, String>>>>
where
    T: Transport + Clone,
    N: Network,
{
    let retry_config = RetryConfig::default();
    retry(
        || async {
            provider
                .debug_trace_block_by_number(block_number, trace_options.clone())
                .await
                .map_err(|e| anyhow!("RPC error: {}", e))
        },
        &retry_config,
        &format!("debug_trace_block_by_number({})", block_number),
    )
    .await
    .map(Some)
}

pub async fn parse_data(
    chain: Chain,
    chain_id: u64,
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
        traces.parse_traces(chain)?
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
    // Only transform data for active datasets, otherwise return empty Vec
    let blocks = if active_datasets.contains(&"blocks".to_string()) {
        parsed_data.clone().transform_blocks(chain)?
    } else {
        vec![]
    };

    let transactions = if active_datasets.contains(&"transactions".to_string())
        && !parsed_data.transactions.is_empty()
    {
        parsed_data.clone().transform_transactions(chain)?
    } else {
        vec![]
    };

    let logs = if active_datasets.contains(&"logs".to_string()) && !parsed_data.logs.is_empty() {
        parsed_data.clone().transform_logs(chain)?
    } else {
        vec![]
    };

    let traces =
        if active_datasets.contains(&"traces".to_string()) && !parsed_data.traces.is_empty() {
            parsed_data.clone().transform_traces(chain)?
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
