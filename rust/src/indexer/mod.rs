// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

pub mod rpc;
pub mod transformations;

use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_network::{primitives::BlockTransactionsKind, Network};
use alloy_provider::{ext::DebugApi, Provider, ReqwestProvider};
use alloy_rpc_types_eth::{Block, TransactionReceipt, Withdrawal};
use alloy_rpc_types_trace::{
    common::TraceResult,
    geth::{GethDebugTracingOptions, GethTrace},
};
use alloy_transport::{RpcError, Transport};

use eyre::Result;
use tracing::error;

use crate::indexer::rpc::blocks::BlockParser;
use crate::indexer::rpc::receipts::ReceiptParser;
use crate::indexer::rpc::traces::TraceParser;
use crate::indexer::transformations::blocks::BlockTransformer;
use crate::indexer::transformations::logs::LogTransformer;
use crate::indexer::transformations::traces::TraceTransformer;
use crate::indexer::transformations::transactions::TransactionTransformer;
use crate::models::common::{ParsedData, TransformedData};

pub async fn get_chain_id(provider: &ReqwestProvider) -> Result<u64> {
    let chain_id = provider.get_chain_id().await?;
    Ok(chain_id)
}

pub async fn get_latest_block_number(provider: &ReqwestProvider) -> Result<BlockNumberOrTag> {
    // TODO: Why do I use ReqwestProvider here?
    let latest_block = provider.get_block_number().await?;
    Ok(BlockNumberOrTag::Number(latest_block)) // TODO: Why do I wrap this but not other results?
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
    let block = provider.get_block_by_number(block_number, kind).await?;
    Ok(block)
}

pub async fn get_block_receipts<T, N>(
    provider: &dyn Provider<T, N>,
    block: BlockId,
) -> Result<Option<Vec<N::ReceiptResponse>>>
where
    T: Transport + Clone,
    N: Network,
{
    let receipts = provider.get_block_receipts(block).await?;
    Ok(receipts)
}

pub async fn debug_trace_block_by_number<T, N>(
    provider: &impl DebugApi<N, T>,
    block_number: BlockNumberOrTag,
    trace_options: GethDebugTracingOptions,
) -> Result<Vec<TraceResult<GethTrace, String>>>
where
    T: Transport + Clone,
    N: Network,
{
    let traces = provider
        .debug_trace_block_by_number(block_number, trace_options)
        .await?;
    Ok(traces)
}

pub async fn parse_data(
    chain_id: u64,
    block: Block,
    receipts: Vec<TransactionReceipt>,
    traces: Vec<TraceResult<GethTrace, String>>,
) -> Result<ParsedData> {
    let header = block.clone().parse_header()?; //TODO: Remove clone
    let transactions = block.clone().parse_transactions()?;
    let withdrawals = block.clone().parse_withdrawals()?;

    let transaction_receipts = receipts.clone().parse_transaction_receipts()?;
    let logs = receipts.clone().parse_log_receipts()?;

    let traces = traces.clone().parse_traces()?;

    Ok(ParsedData {
        chain_id: chain_id,
        header: header,
        transactions: transactions,
        withdrawals: withdrawals,
        transaction_receipts: transaction_receipts,
        logs: logs,
        traces: traces,
    })
}

pub async fn transform_data(parsed_data: ParsedData) -> Result<TransformedData> {
    let blocks = parsed_data.clone().transform_blocks()?;
    let transactions = parsed_data.clone().transform_transactions()?;
    let logs = parsed_data.clone().transform_logs()?;
    let traces = parsed_data.clone().transform_traces()?;

    Ok(TransformedData {
        blocks: blocks,
        transactions: transactions,
        logs: logs,
        traces: traces,
    })
}
