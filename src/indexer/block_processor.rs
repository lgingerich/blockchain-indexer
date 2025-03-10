use alloy_consensus::TxEnvelope;
use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_network::{primitives::BlockTransactionsKind, AnyNetwork, AnyTxEnvelope, Network};
use alloy_primitives::FixedBytes;
use alloy_provider::Provider;
use alloy_rpc_types_trace::geth::{GethDebugBuiltInTracerType, GethDebugTracerConfig, GethDebugTracerType, GethDebugTracingOptions};
use alloy_transport::Transport;
use anyhow::Result;

use crate::metrics::Metrics;
use crate::models::common::{Chain, TransformedData};

pub struct BlockProcessor<T, N>
where
    T: Transport + Clone + Send + Sync + 'static,
    N: Network,
{
    provider: Box<dyn Provider<T, N>>,
    chain: Chain,
    chain_id: u64,
    metrics: Option<Metrics>,
    datasets: Vec<String>,
}

impl<T, N> BlockProcessor<T, N>
where
    T: Transport + Clone + Send + Sync + 'static,
{
    pub fn new(
        provider: Box<dyn Provider<T, N>>,
        chain: Chain,
        chain_id: u64,
        metrics: Option<Metrics>,
        datasets: Vec<String>,
    ) -> Self {
        Self {
            provider,
            chain,
            chain_id,
            metrics,
            datasets,
        }
    }

    pub async fn process_block(&self, block_number: u64) -> Result<ProcessedBlock> {
        let block_number_tag = BlockNumberOrTag::Number(block_number);

        // Track which RPC responses we need
        let need_block = self.datasets.contains(&"blocks".to_string()) 
            || self.datasets.contains(&"transactions".to_string());
        let need_receipts = self.datasets.contains(&"logs".to_string()) 
            || self.datasets.contains(&"transactions".to_string());
        let need_traces = self.datasets.contains(&"traces".to_string());

        // Get block by number if needed
        let block = if need_block {
            let kind = BlockTransactionsKind::Full;
            let provider = &*self.provider;
            super::get_block_by_number(provider, block_number_tag, kind, self.metrics.as_ref())
                .await?
                .ok_or_else(|| anyhow::anyhow!("Provider returned no block"))?
        } else {
            Default::default()
        };

        // Get receipts by block number if needed
        let receipts = if need_receipts {
            let block_id = BlockId::Number(block_number_tag);
            let provider = &*self.provider;
            super::get_block_receipts(provider, block_id, self.metrics.as_ref())
                .await?
                .ok_or_else(|| anyhow::anyhow!("Provider returned no receipts"))?
        } else {
            Vec::new()
        };

        // Get traces if needed
        let traces = if need_traces {
            let tx_hashes: Vec<FixedBytes<32>> = block
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
                .collect();

            let trace_options = GethDebugTracingOptions {
                tracer: Some(GethDebugTracerType::BuiltInTracer(
                    GethDebugBuiltInTracerType::CallTracer,
                )),
                tracer_config: GethDebugTracerConfig(serde_json::json!({"onlyTopCall": false})),
                timeout: Some("10s".to_string()),
                ..Default::default()
            };

            let provider = &*self.provider;
            super::debug_trace_transaction_by_hash(
                provider,
                tx_hashes,
                trace_options,
                self.metrics.as_ref(),
            )
            .await?
            .ok_or_else(|| anyhow::anyhow!("Provider returned no traces"))?
        } else {
            Vec::new()
        };

        // Parse and transform the data
        let parsed_data = super::parse_data(
            self.chain,
            self.chain_id,
            block_number,
            Some(block),
            Some(receipts),
            Some(traces),
        )
        .await?;

        let transformed_data = super::transform_data(self.chain, parsed_data, &self.datasets).await?;

        Ok(ProcessedBlock {
            block_number,
            data: transformed_data,
        })
    }
}

pub struct ProcessedBlock {
    pub block_number: u64,
    pub data: TransformedData,
}