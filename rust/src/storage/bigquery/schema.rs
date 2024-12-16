use google_cloud_bigquery::http::table::{TableFieldSchema, TableFieldType, TableFieldMode, TableSchema};

// TODO: I changed many numbers to strings for easier handling. Is there a better way?

pub fn block_schema() -> TableSchema {
    TableSchema {
        fields: vec![
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
                name: "number".to_string(),
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
                mode: Some(TableFieldMode::Required),
                description: None,
                ..Default::default()
            },
            TableFieldSchema {
                name: "nonce".to_string(),
                data_type: TableFieldType::String,
                mode: Some(TableFieldMode::Required),
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
                name: "target_blobs_per_block".to_string(),
                data_type: TableFieldType::Integer,
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
        ],
    }
}

pub fn transaction_schema() -> TableSchema {
    TableSchema {
        fields: vec![
            TableFieldSchema {
                name: "chain_id".to_string(),
                data_type: TableFieldType::Integer,
                mode: Some(TableFieldMode::Required),
                description: None,
                ..Default::default()
            },
            TableFieldSchema {
                name: "nonce".to_string(),
                data_type: TableFieldType::Integer,
                mode: Some(TableFieldMode::Required),
                description: None,
                ..Default::default()
            },
            TableFieldSchema {
                name: "gas_price".to_string(),
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
                name: "max_fee_per_gas".to_string(),
                data_type: TableFieldType::Integer,
                mode: Some(TableFieldMode::Required),
                description: None,
                ..Default::default()
            },
            TableFieldSchema {
                name: "max_priority_fee_per_gas".to_string(),
                data_type: TableFieldType::Integer,
                mode: Some(TableFieldMode::Required),
                description: None,
                ..Default::default()
            },
            TableFieldSchema {
                name: "value".to_string(),
                data_type: TableFieldType::String,
                mode: Some(TableFieldMode::Required),
                description: None,
                ..Default::default()
            },
            TableFieldSchema {
                name: "access_list".to_string(),
                data_type: TableFieldType::String,
                mode: Some(TableFieldMode::Repeated),
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
            TableFieldSchema {
                name: "max_fee_per_blob_gas".to_string(),
                data_type: TableFieldType::Integer,
                mode: Some(TableFieldMode::Required),
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
            TableFieldSchema {
                name: "input".to_string(),
                data_type: TableFieldType::String,
                mode: Some(TableFieldMode::Required),
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
            // Receipt fields
            TableFieldSchema {
                name: "status".to_string(),
                data_type: TableFieldType::Boolean,
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
                name: "logs_bloom".to_string(),
                data_type: TableFieldType::String,
                mode: Some(TableFieldMode::Required),
                description: None,
                ..Default::default()
            },
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
                name: "authorization_list".to_string(),
                data_type: TableFieldType::String,
                mode: Some(TableFieldMode::Repeated),
                description: None,
                ..Default::default()
            },
        ],
    }
}

pub fn log_schema() -> TableSchema {
    TableSchema {
        fields: vec![
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
        ],
    }
}

pub fn trace_schema() -> TableSchema {
    TableSchema {
        fields: vec![
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
                data_type: TableFieldType::String,
                mode: Some(TableFieldMode::Repeated),
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
        ],
    }
}
