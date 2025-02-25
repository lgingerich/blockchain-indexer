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

//////// Abstract test params ////////
const ABSTRACT_RPC_URL: &str = "https://api.mainnet.abs.xyz";
// ABSTRACT_PARAMS has the block number to process and the expected output row count for each dataset
const ABSTRACT_PARAMS: [(u64, usize, usize, usize, usize); 5] = [
    // (block_number, output_block_count, output_transaction_count, output_log_count, output_trace_count)
    (3, 1, 1, 5, 35),         // First block with a priority (0xff: 255) transaction
    (6, 1, 1, 6, 92),         // First block with an EIP-712 (0x71: 113)
    (8, 1, 1, 4, 66),         // First block with an EIP-1559 (0x2: 2) transaction
    (9, 1, 1, 3, 61),         // First block with a legacy (0x0: 0) transaction
    (165866, 1, 9, 93, 1084), // First block with a type 254 (0xfe: 254) transaction
];

//////// Sophon test params ////////
const SOPHON_RPC_URL: &str = "https://rpc.sophon.xyz";
// SOPHON_PARAMS has the block number to process and the expected output row count for each dataset
const SOPHON_PARAMS: [(u64, usize, usize, usize, usize); 5] = [
    // (block_number, output_block_count, output_transaction_count, output_log_count, output_trace_count)
    (3, 1, 1, 3, 35),         // First block with a priority (0xff: 255) transaction
    (4, 1, 1, 3, 61),         // First block with an EIP-1559 (0x2: 2) transaction
    (6, 1, 1, 7, 99),         // First block with an EIP-712 (0x71: 113)
    (42, 1, 1, 4, 63),        // First block with a legacy (0x0: 0) transaction
    (4044460, 1, 1, 27, 198), // First block with a type 254 (0xfe: 254) transaction
];

/// Process a single chain's test cases
async fn process_chain_test(
    chain: Chain,
    chain_name: &str,
    rpc_url: &str,
    block_cases: Vec<(u64, usize, usize, usize, usize)>,
) -> Result<()> {
    println!("\nTesting {} chain", chain_name);

    // Set up provider
    let provider = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .on_http(rpc_url.parse::<Url>()?);

    // Get chain ID
    let chain_id = indexer::get_chain_id(&provider, None, None).await?;
    assert_eq!(chain, Chain::from_chain_id(chain_id)?);

    for (block_number, expected_blocks, expected_txs, expected_logs, expected_traces) in block_cases
    {
        println!("\nProcessing {} block {}", chain_name, block_number);
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
            indexer::debug_trace_transaction_by_hash(
                &provider,
                tx_hashes,
                trace_options,
                None,
                None,
            )
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
            "{} Block {}: Expected {} blocks, got {}",
            chain_name,
            block_number.as_number().unwrap(),
            expected_blocks,
            transformed_data.blocks.len()
        );

        assert_eq!(
            transformed_data.transactions.len(),
            expected_txs,
            "{} Block {}: Expected {} transactions, got {}",
            chain_name,
            block_number.as_number().unwrap(),
            expected_txs,
            transformed_data.transactions.len()
        );

        assert_eq!(
            transformed_data.logs.len(),
            expected_logs,
            "{} Block {}: Expected {} logs, got {}",
            chain_name,
            block_number.as_number().unwrap(),
            expected_logs,
            transformed_data.logs.len()
        );

        assert_eq!(
            transformed_data.traces.len(),
            expected_traces,
            "{} Block {}: Expected {} traces, got {}",
            chain_name,
            block_number.as_number().unwrap(),
            expected_traces,
            transformed_data.traces.len()
        );

        println!(
            "{} Block {} processed successfully:",
            chain_name,
            block_number.as_number().unwrap()
        );
        println!("- {} blocks", transformed_data.blocks.len());
        println!("- {} transactions", transformed_data.transactions.len());
        println!("- {} logs", transformed_data.logs.len());
        println!("- {} traces", transformed_data.traces.len());
    }

    Ok(())
}

#[tokio::test]
async fn test_indexing_pipeline() -> Result<()> {
    // Test blocks for each chain
    let test_cases = vec![
        // Ethereum blocks
        (
            Chain::Ethereum,
            "Ethereum",
            ETH_RPC_URL,
            ETH_PARAMS.to_vec(),
        ),
        // ZKSync blocks
        (
            Chain::ZKsync,
            "ZKsync Era",
            ZKSYNC_RPC_URL,
            ZKSYNC_PARAMS.to_vec(),
        ),
        // Abstract blocks
        (
            Chain::ZKsync,
            "Abstract",
            ABSTRACT_RPC_URL,
            ABSTRACT_PARAMS.to_vec(),
        ),
        // Sophon blocks
        (
            Chain::ZKsync,
            "Sophon",
            SOPHON_RPC_URL,
            SOPHON_PARAMS.to_vec(),
        ),
    ];

    // Create a vector of futures for each chain test
    let chain_futures = test_cases
        .into_iter()
        .map(|(chain, chain_name, rpc_url, block_cases)| {
            process_chain_test(chain, chain_name, rpc_url, block_cases)
        })
        .collect::<Vec<_>>();

    // Run all chain tests in parallel
    let results = futures::future::join_all(chain_futures).await;

    // Check if any tests failed
    for result in results {
        if let Err(e) = result {
            return Err(e);
        }
    }

    Ok(())
}
