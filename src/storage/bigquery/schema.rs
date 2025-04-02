use google_cloud_bigquery::http::table::{
    TableFieldMode, TableFieldSchema, TableFieldType, TableSchema,
};

use crate::models::common::Chain;

pub fn block_schema(chain: Chain) -> TableSchema {
    let mut fields = vec![
        TableFieldSchema {
            name: "chain_id".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: Some("Chain ID".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_time".to_string(),
            data_type: TableFieldType::Timestamp,
            mode: Some(TableFieldMode::Required),
            description: Some("Timestamp of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_date".to_string(),
            data_type: TableFieldType::Date,
            mode: Some(TableFieldMode::Required),
            description: Some("Date of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_number".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: Some("Number of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Hash of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "parent_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Hash of the parent block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "nonce".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some(
                "Unique value used to prove block computation in PoW consensus".to_string(),
            ),
            ..Default::default()
        },
        TableFieldSchema {
            name: "gas_limit".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: Some("Maximum gas allowed for the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "gas_used".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: Some("Total gas used for the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "base_fee_per_gas".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Base fee per gas in the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "blob_gas_used".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Amount of blob gas used in the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "excess_blob_gas".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Excess blob gas available in the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "extra_data".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Additional data included by miners in the block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "difficulty".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Difficulty of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "total_difficulty".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Cumulative chain difficulty up to this block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "size".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Size of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "miner".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Address of the block rewards beneficiary (miner)".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "logs_bloom".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Bloom filter for the logs of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "sha3_uncles".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Hash of the uncles list for the block".to_string()),
            ..Default::default()
        },        
        TableFieldSchema {
            name: "receipts_root".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Root of the receipts trie of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "state_root".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Root of the state trie of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "transactions_root".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Root of the transactions trie of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "withdrawals_root".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Root of the withdrawals trie of the current block".to_string()),
            ..Default::default()
        },
    ];

    // Add chain-specific fields
    match chain {
        Chain::Ethereum => {
            // No extra fields for Ethereum
        }
        Chain::ZKsync => {
            // ZkSync-specific fields
            fields.extend(vec![
                TableFieldSchema {
                    name: "l1_batch_number".to_string(),
                    data_type: TableFieldType::Integer,
                    mode: Some(TableFieldMode::Nullable),
                    description: Some("Layer 1 batch sequence number".to_string()),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "l1_batch_timestamp".to_string(),
                    data_type: TableFieldType::Timestamp,
                    mode: Some(TableFieldMode::Nullable),
                    description: Some("Timestamp of the Layer 1 batch".to_string()),
                    ..Default::default()
                },
            ]);
        }
    }

    TableSchema { fields }
}

pub fn log_schema(chain: Chain) -> TableSchema {
    let fields = vec![
        TableFieldSchema {
            name: "chain_id".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: Some("Chain ID".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_time".to_string(),
            data_type: TableFieldType::Timestamp,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Timestamp of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_date".to_string(),
            data_type: TableFieldType::Date,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Date of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_number".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Number of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Hash of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "tx_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Hash of the current transaction".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "tx_index".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Index of the current transaction in the block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "log_index".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Index of the current log in the transaction".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "address".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Address of the contract that emitted the log".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "topics".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Repeated),
            description: Some("Topics of the current log".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "data".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Data of the current log".to_string()),
            ..Default::default()
        },
    ];

    // Fields are the same for all chain
    match chain {
        Chain::Ethereum => {
            // No extra fields for Ethereum
        }
        Chain::ZKsync => {
            // No extra fields for ZKsync
        }
    }

    TableSchema { fields }
}

pub fn transaction_schema(chain: Chain) -> TableSchema {
    let mut fields = vec![
        TableFieldSchema {
            name: "chain_id".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: Some("Chain ID".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_time".to_string(),
            data_type: TableFieldType::Timestamp,
            mode: Some(TableFieldMode::Required),
            description: Some("Timestamp of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_date".to_string(),
            data_type: TableFieldType::Date,
            mode: Some(TableFieldMode::Required),
            description: Some("Date of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_number".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Number of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Hash of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "tx_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Hash of the current transaction".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "tx_index".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Index of the current transaction in the block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "tx_type".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: Some("Transaction type".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "status".to_string(),
            data_type: TableFieldType::Boolean,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Transaction status".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "nonce".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Transaction nonce".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "from_address".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Transaction sender".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "to_address".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Transaction receiver".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "contract_address".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Address of the contract".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "input".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Input of the current transaction".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "value".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Value of the current transaction".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "gas_price".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Gas price of the current transaction".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "gas_limit".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: Some("Gas limit of the current transaction".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "gas_used".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: Some("Gas used by the current transaction".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "max_fee_per_gas".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Maximum total fee per unit of gas willing to be paid".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "max_priority_fee_per_gas".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some(
                "Maximum tip per unit of gas willing to be paid to validator".to_string(),
            ),
            ..Default::default()
        },
        TableFieldSchema {
            name: "effective_gas_price".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: Some("Final gas price paid per unit of gas".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "cumulative_gas_used".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: Some("Total gas used in the block up to this transaction".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "blob_gas_price".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Price per unit of blob gas for the transaction".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "blob_gas_used".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Amount of blob gas used by the transaction".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "access_list".to_string(),
            data_type: TableFieldType::Record,
            mode: Some(TableFieldMode::Repeated),
            fields: Some(vec![
                TableFieldSchema {
                    name: "address".to_string(),
                    data_type: TableFieldType::String,
                    mode: Some(TableFieldMode::Required),
                    description: Some("Address being accessed".to_string()),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "storage_keys".to_string(),
                    data_type: TableFieldType::String,
                    mode: Some(TableFieldMode::Repeated),
                    description: Some("Storage slots being accessed".to_string()),
                    ..Default::default()
                },
            ]),
            description: Some(
                "List of addresses and storage keys accessed by the transaction".to_string(),
            ),
            ..Default::default()
        },
        TableFieldSchema {
            name: "authorization_list".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Repeated),
            description: Some("List of authorized addresses for the transaction".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "blob_versioned_hashes".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Repeated),
            description: Some(
                "Version hashes of the blobs included in the transaction".to_string(),
            ),
            ..Default::default()
        },
        TableFieldSchema {
            name: "logs_bloom".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Bloom filter containing all transaction logs".to_string()),
            ..Default::default()
        },
    ];

    // Add chain-specific fields
    match chain {
        Chain::Ethereum => {
            fields.extend(vec![
                TableFieldSchema {
                    name: "max_fee_per_blob_gas".to_string(),
                    data_type: TableFieldType::Integer,
                    mode: Some(TableFieldMode::Nullable),
                    description: Some("Maximum fee per blob gas willing to be paid".to_string()),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "blobs".to_string(),
                    data_type: TableFieldType::String,
                    mode: Some(TableFieldMode::Repeated),
                    description: Some("Blob data associated with the transaction".to_string()),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "commitments".to_string(),
                    data_type: TableFieldType::String,
                    mode: Some(TableFieldMode::Repeated),
                    description: Some("KZG commitments for the transaction blobs".to_string()),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "proofs".to_string(),
                    data_type: TableFieldType::String,
                    mode: Some(TableFieldMode::Repeated),
                    description: Some("KZG proofs for the transaction blobs".to_string()),
                    ..Default::default()
                },
            ]);
        }
        Chain::ZKsync => {
            // ZkSync-specific fields
            fields.extend(vec![
                TableFieldSchema {
                    name: "l1_batch_number".to_string(),
                    data_type: TableFieldType::Integer,
                    mode: Some(TableFieldMode::Nullable),
                    description: Some("Batch number on Layer 1".to_string()),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "l1_batch_tx_index".to_string(),
                    data_type: TableFieldType::Integer,
                    mode: Some(TableFieldMode::Nullable),
                    description: Some("Transaction index within the Layer 1 batch".to_string()),
                    ..Default::default()
                },
            ]);
        }
    }

    TableSchema { fields }
}

pub fn trace_schema(chain: Chain) -> TableSchema {
    let fields = vec![
        TableFieldSchema {
            name: "chain_id".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: Some("Chain ID".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_time".to_string(),
            data_type: TableFieldType::Timestamp,
            mode: Some(TableFieldMode::Required),
            description: Some("Timestamp of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_date".to_string(),
            data_type: TableFieldType::Date,
            mode: Some(TableFieldMode::Required),
            description: Some("Date of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_number".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Number of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Hash of the current block".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "tx_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Hash of the transaction containing this trace".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "tx_index".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Index of the transaction containing this trace".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "trace_type".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Trace operation type".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "subtraces".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: Some("Number of subtraces in the trace".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "trace_address".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Repeated),
            description: Some(
                "Array of indices defining the trace's position in the call tree".to_string(),
            ),
            ..Default::default()
        },
        TableFieldSchema {
            name: "from_address".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Address initiating the trace call".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "to_address".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Address receiving the trace call".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "value".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Value transferred in the trace call".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "gas".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Gas allocated for the trace execution".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "gas_used".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Gas consumed by the trace execution".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "input".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: Some("Input data for the trace call".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "output".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Output data from the trace execution".to_string()),
            ..Default::default()
        },
        TableFieldSchema {
            name: "error".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: Some("Error message if the trace execution failed".to_string()),
            ..Default::default()
        },
    ];

    // Fields are the same for all chain
    match chain {
        Chain::Ethereum => {
            // No extra fields for Ethereum
        }
        Chain::ZKsync => {
            // No extra fields for ZKsync
        }
    }

    TableSchema { fields }
}
