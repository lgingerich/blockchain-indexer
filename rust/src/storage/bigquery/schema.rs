use google_cloud_bigquery::http::table::{
    TableFieldMode, TableFieldSchema, TableFieldType, TableSchema,
};

use crate::models::common::Chain;
use anyhow::{anyhow, Result};


pub async fn create_schema(chain: Chain, table_name: &str) -> Result<TableSchema> {
    match table_name {
        "blocks" => Ok(block_schema(chain)),
        "logs" => Ok(log_schema(chain)),
        "transactions" => Ok(transaction_schema(chain)),
        "traces" => Ok(trace_schema(chain)),
        _ => Err(anyhow!("Invalid table name: {}", table_name)),
    }
}

pub fn block_schema(chain: Chain) -> TableSchema {
    let mut fields = vec![
        // Common fields that exist for all chains
        TableFieldSchema {
            name: "chain_id".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "parent_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "ommers_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "beneficiary".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "state_root".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "transactions_root".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "receipts_root".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "logs_bloom".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "difficulty".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_number".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "gas_limit".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "gas_used".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_time".to_string(),
            data_type: TableFieldType::Timestamp,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_date".to_string(),
            data_type: TableFieldType::Date,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "extra_data".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "mix_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "nonce".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },        
        TableFieldSchema {
            name: "base_fee_per_gas".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "withdrawals_root".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "blob_gas_used".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "excess_blob_gas".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "parent_beacon_block_root".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "requests_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "total_difficulty".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "size".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
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
                    name: "target_blobs_per_block".to_string(),
                    data_type: TableFieldType::Integer,
                    mode: Some(TableFieldMode::Nullable),
                    description: None,
                    ..Default::default()
                },                
                TableFieldSchema {
                    name: "l1_batch_number".to_string(),
                    data_type: TableFieldType::Integer,
                    mode: Some(TableFieldMode::Nullable),
                    description: None,
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "l1_batch_timestamp".to_string(),
                    data_type: TableFieldType::Timestamp,
                    mode: Some(TableFieldMode::Nullable),
                    description: None,
                    ..Default::default()
                },
                // TableFieldSchema { // TODO: Add this back in
                //     name: "seal_fields".to_string(),
                //     data_type: TableFieldType::String,
                //     mode: Some(TableFieldMode::Repeated),
                //     description: None,
                //     ..Default::default()
                // },
            ]);
        }
    }

    TableSchema { fields }
}

// Implement similar chain-aware schemas for other tables
pub fn log_schema(chain: Chain) -> TableSchema {
    let mut fields = vec![
        TableFieldSchema {
            name: "address".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "topics".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Repeated),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "data".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_number".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_timestamp".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "transaction_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "transaction_index".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "log_index".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "removed".to_string(),
            data_type: TableFieldType::Boolean,
            mode: Some(TableFieldMode::Required),
            description: None,
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
        // Block fields
        TableFieldSchema {
            name: "chain_id".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "tx_type".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "nonce".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "gas_price".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "gas_limit".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "max_fee_per_gas".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "max_priority_fee_per_gas".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "value".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
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
                    description: None,
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "storage_keys".to_string(),
                    data_type: TableFieldType::String,
                    mode: Some(TableFieldMode::Repeated),
                    description: None,
                    ..Default::default()
                },
            ]),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "input".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },        
        TableFieldSchema {
            name: "r".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "s".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "v".to_string(),
            data_type: TableFieldType::Boolean,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "blob_versioned_hashes".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Repeated),
            description: None,
            ..Default::default()
        },

        // Receipt fields
        TableFieldSchema {
            name: "transaction_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "transaction_index".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },        
        TableFieldSchema {
            name: "status".to_string(),
            data_type: TableFieldType::Boolean,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_hash".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "block_number".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },        
        TableFieldSchema {
            name: "gas_used".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },        
        TableFieldSchema {
            name: "effective_gas_price".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "blob_gas_used".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "blob_gas_price".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "from".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "to".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "contract_address".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },        
        TableFieldSchema {
            name: "cumulative_gas_used".to_string(),
            data_type: TableFieldType::Integer,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "authorization_list".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Repeated),
            description: None,
            ..Default::default()
        },        
        TableFieldSchema {
            name: "logs_bloom".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
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
                    description: None,
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "blobs".to_string(),
                    data_type: TableFieldType::String,
                    mode: Some(TableFieldMode::Repeated),
                    description: None,
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "commitments".to_string(),
                    data_type: TableFieldType::String,
                    mode: Some(TableFieldMode::Repeated),
                    description: None,
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "proofs".to_string(),
                    data_type: TableFieldType::String,
                    mode: Some(TableFieldMode::Repeated),
                    description: None,
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
                    description: None,
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "l1_batch_tx_index".to_string(),
                    data_type: TableFieldType::Integer,
                    mode: Some(TableFieldMode::Nullable),
                    description: None,
                    ..Default::default()
                },  
            ]);
        }
    }

    TableSchema { fields }
}

pub fn trace_schema(chain: Chain) -> TableSchema {
    let mut fields = vec![
        TableFieldSchema {
            name: "from".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "gas".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "gas_used".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "to".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "input".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "output".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "error".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "revert_reason".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "logs".to_string(),
            data_type: TableFieldType::Record,
            mode: Some(TableFieldMode::Repeated),
            fields: Some(vec![
                TableFieldSchema {
                    name: "address".to_string(),
                    data_type: TableFieldType::String,
                    mode: Some(TableFieldMode::Nullable),
                    description: None,
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "topics".to_string(),
                    data_type: TableFieldType::String,
                    mode: Some(TableFieldMode::Repeated),
                    description: None,
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "data".to_string(),
                    data_type: TableFieldType::String,
                    mode: Some(TableFieldMode::Nullable),
                    description: None,
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "position".to_string(),
                    data_type: TableFieldType::Integer,
                    mode: Some(TableFieldMode::Nullable),
                    description: None,
                    ..Default::default()
                },
            ]),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "value".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Nullable),
            description: None,
            ..Default::default()
        },
        TableFieldSchema {
            name: "typ".to_string(),
            data_type: TableFieldType::String,
            mode: Some(TableFieldMode::Required),
            description: None,
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