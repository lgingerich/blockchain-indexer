mod schema;

use crate::metrics::Metrics;
use crate::models::common::Chain;
use crate::storage::bigquery::schema::{
    block_schema, log_schema, trace_schema, transaction_schema,
};
use crate::utils::retry::{retry, RetryConfig};
use anyhow::{anyhow, Result};
use google_cloud_bigquery::client::{Client, ClientConfig};
use google_cloud_bigquery::http::dataset::{Dataset, DatasetReference};
use google_cloud_bigquery::http::error::Error as BigQueryError;
use google_cloud_bigquery::http::job::query::QueryRequest;
use google_cloud_bigquery::http::table::{
    Table, TableReference, TimePartitionType, TimePartitioning,
};
use google_cloud_bigquery::http::tabledata::{
    insert_all::{InsertAllRequest, Row as TableRow},
    list::Value,
};
use once_cell::sync::OnceCell;
use opentelemetry::KeyValue;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::{error, info, warn};

// Define a static OnceCell to hold the shared Client and Project ID
static BIGQUERY_CLIENT: OnceCell<Arc<(Client, String)>> = OnceCell::new();

// Initializes and returns the shared BigQuery Client and Project ID.
// This function ensures that the Client is initialized only once.
pub async fn get_client() -> Result<Arc<(Client, String)>> {
    if let Some(client) = BIGQUERY_CLIENT.get() {
        return Ok(client.clone());
    }

    let (config, project_id_option) = ClientConfig::new_with_auth().await?;
    let client = Client::new(config).await?;
    let project_id = project_id_option.ok_or_else(|| anyhow!("Project ID not found"))?;
    info!("Project ID: {}", project_id);

    let client_arc = Arc::new((client, project_id));

    match BIGQUERY_CLIENT.set(client_arc.clone()) {
        Ok(()) => Ok(client_arc),
        Err(_e) => {
            // If we failed to set (because another thread set it first),
            // return the value that was set by the other thread
            Ok(BIGQUERY_CLIENT.get().unwrap().clone())
        }
    }
}

// Verify that a dataset exists and is accessible
pub async fn verify_dataset(client: &Client, project_id: &str, chain_name: &str) -> Result<bool> {
    match client.dataset().get(project_id, chain_name).await {
        Ok(_) => Ok(true),
        Err(BigQueryError::Response(resp)) if resp.message.contains("Not found") => Ok(false),
        Err(e) => Err(anyhow!("Failed to verify dataset: {}", e)),
    }
}

// Verify that a table exists and is accessible
pub async fn verify_table(
    client: &Client,
    project_id: &str,
    chain_name: &str,
    table_id: &str,
) -> Result<bool> {
    match client.table().get(project_id, chain_name, table_id).await {
        Ok(_) => Ok(true),
        Err(BigQueryError::Response(resp)) if resp.message.contains("Not found") => Ok(false),
        Err(e) => Err(anyhow!("Failed to verify table: {}", e)),
    }
}

// Create a dataset
pub async fn create_dataset(chain_name: &str) -> Result<()> {
    let (client, project_id) = &*get_client().await?;
    let dataset_client = client.dataset();

    // Check if dataset exists first
    if verify_dataset(client, project_id, chain_name).await? {
        info!("Dataset '{}' already exists and is accessible", chain_name);
        return Ok(());
    }

    let metadata = Dataset {
        dataset_reference: DatasetReference {
            project_id: project_id.clone(),
            dataset_id: chain_name.to_string(),
        },
        ..Default::default()
    };

    // Retry::spawn(get_retry_config("create_dataset"), || async {
    let retry_config = RetryConfig::default();
    retry(
        || async {
            match dataset_client.create(&metadata).await {
                Ok(_) => {
                    info!(chain_name, project_id = ?project_id, "Dataset successfully created");
                    Ok(())
                }
                Err(BigQueryError::Response(resp)) if resp.message.contains("Already Exists") => {
                    info!("Dataset '{}' already exists", chain_name);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to create dataset: {}", e);
                    Err(anyhow!("Dataset creation failed: {}", e))
                }
            }
        },
        &retry_config,
        "create_dataset",
    )
    .await
    .map_err(|e| anyhow!("Failed to create dataset after retries: {}", e))?;

    Ok(())
}

// Create a table
pub async fn create_table(chain_name: &str, table_id: &str, chain: Chain) -> Result<()> {
    let (client, project_id) = &*get_client().await?;
    let table_client = client.table(); // Create BigqueryTableClient

    // Check if table exists
    if verify_table(client, project_id, chain_name, table_id).await? {
        info!(
            "Table '{}.{}' already exists and is accessible",
            chain_name, table_id
        );
        return Ok(());
    }

    let schema = match table_id {
        "blocks" => block_schema(chain),
        "logs" => log_schema(chain),
        "transactions" => transaction_schema(chain),
        "traces" => trace_schema(chain),
        _ => return Err(anyhow!("Invalid table ID: {}", table_id)),
    };

    let metadata = Table {
        table_reference: TableReference {
            project_id: project_id.clone(),
            dataset_id: chain_name.to_string(),
            table_id: table_id.to_string(),
        },
        schema: Some(schema),
        time_partitioning: Some(TimePartitioning {
            partition_type: TimePartitionType::Day,
            field: Some("block_date".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Retry::spawn(get_retry_config("create_table"), || async {
    let retry_config = RetryConfig::default();
    retry(
        || async {
            match table_client.create(&metadata).await {
                Ok(_) => {
                    info!(
                        "Table '{}' successfully created in dataset '{}'",
                        table_id, chain_name
                    );
                    Ok(())
                }
                Err(e) => {
                    match e {
                        BigQueryError::Response(resp) => {
                            if resp.message.contains("Already Exists") {
                                info!(
                                    "Table '{}' already exists in dataset '{}'",
                                    table_id, chain_name
                                );
                                return Ok(());
                            }
                            error!("BigQuery API Error: {}", resp.message);
                        }
                        BigQueryError::HttpClient(e) => {
                            error!("HTTP Client error: {}", e);
                        }
                        BigQueryError::HttpMiddleware(e) => {
                            error!("HTTP Middleware error: {}", e);
                        }
                        BigQueryError::TokenSource(e) => {
                            error!("Token Source error: {}", e);
                        }
                    }
                    Err(anyhow!("Table creation failed"))
                }
            }
        },
        &retry_config,
        "create_table",
    )
    .await
    .map_err(|e| anyhow!("Failed to create table after retries: {}", e))?;

    Ok(())
}

// Insert data into a table
pub async fn insert_data<T: serde::Serialize>(
    chain_name: &str,
    table_id: &str,
    data: Vec<T>,
    block_range: (u64, u64),
    metrics: Option<&Metrics>,
) -> Result<()> {
    let (client, project_id) = &*get_client().await?;

    if data.is_empty() {
        info!(
            "No data to insert into {}.{}.{} for blocks {} to {}",
            project_id, chain_name, table_id, block_range.0, block_range.1
        );
        return Ok(());
    }

    let total_rows = data.len();
    let batch_start = std::time::Instant::now();

    // Record batch size if metrics enabled
    if let Some(metrics) = metrics {
        metrics.bigquery_batch_size.record(
            total_rows as f64,
            &[
                KeyValue::new("chain", metrics.chain_name.clone()),
                KeyValue::new("table", table_id.to_string()),
            ],
        );
    }

    let mut current_batch = Vec::new();
    let mut current_size: usize = 0;
    let mut batches_sent = 0;

    // BigQuery hard limit & safety margins
    const BQ_MAX_BYTES: usize = 10_000_000; // 10 MiB
    const SAFETY_MARGIN: usize = 512_000; // 0.5 MiB head room
    const MAX_BATCH_BYTES: usize = BQ_MAX_BYTES - SAFETY_MARGIN; // 9.5 MiB effective
    const ROW_OVERHEAD: usize = 200; // rough JSON envelope per row

    // Async helper that sends the accumulated batch and resets the counters.
    async fn flush_batch<T: serde::Serialize>(
        client: &Client,
        project_id: &str,
        chain_name: &str,
        table_id: &str,
        batch: &mut Vec<T>,
        block_range: (u64, u64),
    ) -> Result<bool> {
        if batch.is_empty() {
            return Ok(false); // nothing sent
        }

        let rows: Vec<TableRow<&T>> = batch
            .iter()
            .map(|item| TableRow {
                insert_id: Some(generate_insert_id(table_id, item, block_range.0)),
                json: item,
            })
            .collect();

        let request = InsertAllRequest {
            skip_invalid_rows: Some(true),
            ignore_unknown_values: Some(true),
            template_suffix: None,
            rows,
            trace_id: None,
        };

        let retry_config = RetryConfig::default();
        retry(
            || async {
                match client
                    .tabledata()
                    .insert(project_id, chain_name, table_id, &request)
                    .await
                {
                    Ok(response) => {
                        if let Some(errors) = response.insert_errors {
                            if !errors.is_empty() {
                                for error in errors {
                                    error!("Row {} failed to insert", error.index);
                                    for err_msg in error.errors {
                                        error!("Error: {}", err_msg.message);
                                    }
                                }
                                return Err(anyhow!("Some rows failed to insert"));
                            }
                        }
                        Ok(())
                    }
                    Err(e) => {
                        match e {
                            BigQueryError::Response(resp) => {
                                error!("BigQuery API Error: {}", resp.message)
                            }
                            BigQueryError::HttpClient(e) => error!("HTTP Client error: {}", e),
                            BigQueryError::HttpMiddleware(e) => {
                                error!("HTTP Middleware error: {}", e)
                            }
                            BigQueryError::TokenSource(e) => error!("Token Source error: {}", e),
                        }
                        Err(anyhow!("Data insertion failed"))
                    }
                }
            },
            &retry_config,
            "insert_data",
        )
        .await?;

        batch.clear();
        Ok(true) // something was sent
    }

    for item in data {
        // Estimate row size (payload + overhead)
        let item_json_str = serde_json::to_string(&item)?;
        let estimated_size = item_json_str.len() + ROW_OVERHEAD;

        // If adding this row would exceed limit, flush first
        if current_size + estimated_size > MAX_BATCH_BYTES {
            if flush_batch(
                client,
                project_id,
                chain_name,
                table_id,
                &mut current_batch,
                block_range,
            )
            .await?
            {
                batches_sent += 1;
            }
            current_size = 0;
        }

        // Handle row larger than max on its own
        if estimated_size > MAX_BATCH_BYTES {
            // Send it as a single row batch
            let mut single = vec![item];
            flush_batch(
                client,
                project_id,
                chain_name,
                table_id,
                &mut single,
                block_range,
            )
            .await?;
            batches_sent += 1;
            continue;
        }

        current_size += estimated_size;
        current_batch.push(item);
    }

    // Flush remaining rows
    if flush_batch(
        client,
        project_id,
        chain_name,
        table_id,
        &mut current_batch,
        block_range,
    )
    .await?
    {
        batches_sent += 1;
    }

    // After batches are sent, record metrics
    if let Some(metrics) = metrics {
        metrics.bigquery_insert_latency.record(
            batch_start.elapsed().as_secs_f64(),
            &[
                KeyValue::new("chain", metrics.chain_name.clone()),
                KeyValue::new("table", table_id.to_string()),
            ],
        );
    }

    info!(
        "Successfully inserted {} rows into {}.{}.{} for blocks {} to {} in {} batches (took {:.2?})",
        total_rows,
        project_id,
        chain_name,
        table_id,
        block_range.0,
        block_range.1,
        batches_sent + 1,
        batch_start.elapsed()
    );

    Ok(())
}

// Helper function to generate appropriate InsertIDs based on table type and data content
fn generate_insert_id<T: serde::Serialize>(
    table_id: &str,
    data: &T,
    fallback_block_number: u64,
) -> String {
    // First convert the data to a Value so we can access its fields
    let value = match serde_json::to_value(data) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to serialize data for InsertID generation: {}", e);
            // If serialization fails, fall back to a simple block-based ID
            return format!("{table_id}-{fallback_block_number}");
        }
    };

    // Get block_number from the data, with fallback to the parameter
    let block_number = value
        .get("block_number")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_else(|| {
            if table_id != "blocks" {
                // For blocks, block_number might be optional in some cases
                warn!(
                    "Missing block_number in data, using fallback: {}",
                    fallback_block_number
                );
            }
            fallback_block_number
        });

    // Generate a base ID string that might exceed the length limit
    let base_id = match table_id {
        "blocks" => {
            // For blocks, just use the block number
            format!("block-{block_number}")
        }
        "transactions" => {
            // For transactions, combine block_number and tx_hash
            let Some(tx_hash) = value.get("tx_hash").and_then(|v| v.as_str()) else {
                error!(
                    "Missing mandatory tx_hash field in transaction data for block {}",
                    block_number
                );
                return format!("tx-{block_number}-unknown");
            };

            format!("tx-{block_number}-{tx_hash}")
        }
        "logs" => {
            // For logs, combine block_number, tx_hash, tx_index, and log_index
            let Some(tx_hash) = value.get("tx_hash").and_then(serde_json::Value::as_str) else {
                error!(
                    "Missing mandatory tx_hash field in log data for block {}",
                    block_number
                );
                return format!("log-{block_number}-unknown-0-0");
            };

            let Some(tx_index) = value.get("tx_index").and_then(serde_json::Value::as_u64) else {
                error!(
                    "Missing mandatory tx_index field in log data for block {}",
                    block_number
                );
                return format!("log-{block_number}-unknown-0-0");
            };

            let Some(log_index) = value.get("log_index").and_then(serde_json::Value::as_u64) else {
                error!(
                    "Missing mandatory log_index field in log data for block {}",
                    block_number
                );
                return format!("log-{block_number}-unknown-0-0");
            };

            format!("log-{block_number}-{tx_hash}-{tx_index}-{log_index}")
        }
        "traces" => {
            // For traces, combine block_number, tx_hash, and trace_address
            let Some(tx_hash) = value.get("tx_hash").and_then(|v| v.as_str()) else {
                error!(
                    "Missing mandatory tx_hash field in trace data for block {}",
                    block_number
                );
                return format!("trace-{block_number}-unknown-root");
            };

            // Handle trace_address which is an array
            let Some(addr_array) = value.get("trace_address").and_then(|v| v.as_array()) else {
                error!(
                    "Missing mandatory trace_address field in trace data for block {}",
                    block_number
                );
                return format!("trace-{block_number}-unknown-root");
            };

            let trace_address = addr_array
                .iter()
                .map(|v| v.as_u64().unwrap_or(0).to_string())
                .collect::<Vec<String>>()
                .join("-");

            format!("trace-{block_number}-{tx_hash}-{trace_address}")
        }
        // For any other table types
        _ => {
            warn!("Invalid table ID: {}", table_id);
            format!("{table_id}-{block_number}")
        }
    };

    // Check if the base ID exceeds the length limit (128 bytes)
    // UTF-8 characters can be up to 4 bytes each, so we'll be conservative
    if base_id.len() > 120 {
        // Leave some margin for safety
        // If it's too long, hash it to create a fixed-length ID
        // Use the first 16 bytes of the SHA-256 hash (32 hex chars)
        // and prepend with the table ID and block number for readability
        let mut hasher = Sha256::new();
        hasher.update(base_id.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        format!("{table_id}-{block_number}-{}", &hash[..32])
    } else {
        // If it's within the limit, use the base ID as is
        base_id
    }
}

// Get the last processed block number from storage
pub async fn get_last_processed_block(chain_name: &str, datasets: &Vec<String>) -> Result<u64> {
    let (client, project_id) = &*get_client().await?;
    let job_client = client.job(); // Create BigqueryJobClient
    let mut min_block: Option<u64> = None;

    for table_id in datasets {
        // Skip tables that don't exist
        if !verify_table(client, project_id, chain_name, table_id).await? {
            continue;
        }

        let query = format!(
            "SELECT MAX(block_number) AS max_block FROM `{project_id}.{chain_name}.{table_id}`",
        );
        let request = QueryRequest {
            query,
            ..Default::default()
        };
        match job_client.query(project_id, &request).await {
            Ok(result) => {
                if let Some(rows) = result.rows {
                    if !rows.is_empty() {
                        if let Value::String(str_value) = &rows[0].f[0].v {
                            if let Ok(block_num) = str_value.parse::<u64>() {
                                min_block = Some(match min_block {
                                    Some(current_min) => current_min.min(block_num),
                                    None => block_num,
                                });
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to query table {}: {}", table_id, e);
            }
        }
    }
    let min_block = min_block.unwrap_or(0);
    info!("Last processed block: {}", min_block);
    Ok(min_block)
}
