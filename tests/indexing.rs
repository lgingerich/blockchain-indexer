use alloy_consensus::TxEnvelope;
use alloy_eips::BlockNumberOrTag;
use alloy_network::{AnyNetwork, AnyTxEnvelope};
use alloy_primitives::FixedBytes;
use alloy_provider::ProviderBuilder;
use alloy_rpc_types_trace::geth::{
    GethDebugBuiltInTracerType, GethDebugTracerConfig, GethDebugTracerType,
    GethDebugTracingOptions, GethDefaultTracingOptions,
};
use anyhow::Result;
use url::Url;

use blockchain_indexer::{indexer, models::common::Chain};

//////// Ethereum test params ////////
const ETH_RPC_URL: &str = "https://eth.drpc.org";
// ETH_PARAMS has the block number to process and the expected output row count for each dataset
const ETH_PARAMS: [(u64, usize, usize, usize, usize); 4] = [
    // (block_number, output_block_count, output_transaction_count, output_log_count, output_trace_count)
    (46147, 1, 1, 0, 1),           // First block with a legacy transaction
    (12244145, 1, 167, 332, 760),  // First block with an EIP-2930 transaction
    (12965001, 1, 257, 570, 1853), // First block with an EIP-1559 transaction
    (19426589, 1, 79, 205, 506),   // First block with an EIP-4844 transaction
];

//////// ZKsync Era test params ////////
const ZKSYNC_RPC_URL: &str = "https://mainnet.era.zksync.io";
// ZKSYNC_PARAMS has the block number to process and the expected output row count for each dataset
const ZKSYNC_PARAMS: [(u64, usize, usize, usize, usize); 4] = [
    // (block_number, output_block_count, output_transaction_count, output_log_count, output_trace_count)
    (1, 1, 6, 24, 0), // First block with a priority (0xff: 255) transaction
    (13, 1, 2, 8, 0), // First block with an EIP-712 (0x71: 113) and EIP-1559 (0x2: 2) transaction
    (14, 1, 1, 3, 0), // First block with a legacy (0x0: 0) transaction
    (12464133, 1, 41, 250, 3893), // First block with a type 254 (0xfe: 254) transaction
];

#[tokio::test]
async fn test_indexing_pipeline() -> Result<()> {
    // Test blocks for each chain
    let test_cases = vec![
        // Ethereum blocks
        (Chain::Ethereum, ETH_RPC_URL, ETH_PARAMS),
        // ZKSync blocks
        (Chain::ZKsync, ZKSYNC_RPC_URL, ZKSYNC_PARAMS),
    ];

    for (chain, rpc_url, block_cases) in test_cases {
        println!("\nTesting {:?} chain", chain);

        // Set up provider
        let provider = ProviderBuilder::new()
            .network::<AnyNetwork>()
            .on_http(rpc_url.parse::<Url>()?);

        // Get chain ID
        let chain_id = indexer::get_chain_id(&provider, None, None).await?;
        assert_eq!(chain, Chain::from_chain_id(chain_id)?);

        for (block_number, expected_blocks, expected_txs, expected_logs, expected_traces) in
            block_cases
        {
            println!("\nProcessing block {}", block_number);
            let block_number = BlockNumberOrTag::Number(block_number);

            // Fetch all data
            let block = Some(
                indexer::get_block_by_number(
                    &provider,
                    block_number,
                    alloy_network::primitives::BlockTransactionsKind::Full,
                    None,
                    None,
                )
                .await?
                .expect("block should exist"),
            );

            let receipts = Some(
                indexer::get_block_receipts(&provider, block_number.into(), None, None)
                    .await?
                    .expect("receipts should exist"),
            );

            let trace_options = GethDebugTracingOptions {
                config: GethDefaultTracingOptions::default(),
                tracer: Some(GethDebugTracerType::BuiltInTracer(
                    GethDebugBuiltInTracerType::CallTracer,
                )),
                tracer_config: GethDebugTracerConfig(serde_json::json!({"onlyTopCall": false})),
                timeout: Some("10s".to_string()),
            };
            let tx_hashes: Vec<FixedBytes<32>> = if let Some(block_data) = &block {
                block_data
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
                    .collect()
            } else {
                Vec::new()
            };
            let traces = Some(
                indexer::debug_trace_transaction_by_hash(&provider, tx_hashes, trace_options, None, None)
                    .await?
                    .expect("traces should exist"),
            );

            // Parse the raw data
            let parsed_data = indexer::parse_data(
                chain,
                chain_id,
                block_number.as_number().unwrap(),
                block,
                receipts,
                traces,
            )
            .await?;

            // Transform the parsed data
            let datasets = vec![
                "blocks".to_string(),
                "transactions".to_string(),
                "logs".to_string(),
                "traces".to_string(),
            ];

            let transformed_data = indexer::transform_data(chain, parsed_data, &datasets).await?;

            // Verify the transformed data matches expected counts
            assert_eq!(
                transformed_data.blocks.len(),
                expected_blocks,
                "Block {}: Expected {} blocks, got {}",
                block_number.as_number().unwrap(),
                expected_blocks,
                transformed_data.blocks.len()
            );

            assert_eq!(
                transformed_data.transactions.len(),
                expected_txs,
                "Block {}: Expected {} transactions, got {}",
                block_number.as_number().unwrap(),
                expected_txs,
                transformed_data.transactions.len()
            );

            assert_eq!(
                transformed_data.logs.len(),
                expected_logs,
                "Block {}: Expected {} logs, got {}",
                block_number.as_number().unwrap(),
                expected_logs,
                transformed_data.logs.len()
            );

            assert_eq!(
                transformed_data.traces.len(),
                expected_traces,
                "Block {}: Expected {} traces, got {}",
                block_number.as_number().unwrap(),
                expected_traces,
                transformed_data.traces.len()
            );

            println!(
                "Block {} processed successfully:",
                block_number.as_number().unwrap()
            );
            println!("- {} blocks", transformed_data.blocks.len());
            println!("- {} transactions", transformed_data.transactions.len());
            println!("- {} logs", transformed_data.logs.len());
            println!("- {} traces", transformed_data.traces.len());
        }
    }

    Ok(())
}
