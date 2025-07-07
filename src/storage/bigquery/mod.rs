mod schema;

use anyhow::Result;
use google_cloud_bigquery::client::{Client, ClientConfig};
use google_cloud_bigquery::http::{
    dataset::{Dataset, DatasetReference},
    error::Error as BigQueryError,
    job::query::QueryRequest,
    table::{Table as BigQueryTable, TableReference, TimePartitionType, TimePartitioning},
    tabledata::{
        insert_all::{InsertAllRequest, Row as TableRow},
        list::Value,
    },
};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::metrics::Metrics;
use crate::models::common::ChainInfo;
use crate::storage::bigquery::schema::{
    block_schema, log_schema, trace_schema, transaction_schema,
};
use crate::utils::{
    Table,
    retry::{RetryConfig, retry},
};

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
    let project_id = project_id_option.ok_or_else(|| anyhow::anyhow!("Project ID not found"))?;

    let client_arc = Arc::new((client, project_id));

    match BIGQUERY_CLIENT.set(client_arc.clone()) {
        Ok(()) => {
            info!(
                "Initialized and cached BigQuery client for Project ID: {}",
                client_arc.1
            );
            Ok(client_arc)
        }
        Err(_e) => Ok(BIGQUERY_CLIENT.get().unwrap().clone()),
    }
}

// Verify that a dataset exists and is accessible
pub async fn verify_dataset(
    client: &Client,
    project_id: &str,
    chain_info: &ChainInfo,
) -> Result<bool> {
    // TODO: Better handle case when dataset is not found
    match client.dataset().get(project_id, &chain_info.name).await {
        Ok(_) => Ok(true),
        Err(BigQueryError::Response(resp)) if resp.message.contains("Not found") => Ok(false),
        Err(e) => Err(e.into()),
    }
}

// Verify that a table exists and is accessible
pub async fn verify_table(
    client: &Client,
    project_id: &str,
    chain_info: &ChainInfo,
    table_id: &str,
) -> Result<bool> {
    // TODO: Better handle case when table is not found
    match client
        .table()
        .get(project_id, &chain_info.name, table_id)
        .await
    {
        Ok(_) => Ok(true),
        Err(BigQueryError::Response(resp)) if resp.message.contains("Not found") => Ok(false),
        Err(e) => Err(e.into()),
    }
}

// Create a dataset
pub async fn create_dataset(chain_info: &ChainInfo, dataset_location: &str) -> Result<()> {
    let (client, project_id) = &*get_client().await?;
    let dataset_client = client.dataset();

    // Check if dataset exists first
    if verify_dataset(client, project_id, chain_info).await? {
        info!(
            "Dataset '{}' already exists and is accessible",
            chain_info.name
        );
        return Ok(());
    }

    let metadata = Dataset {
        dataset_reference: DatasetReference {
            project_id: project_id.clone(),
            dataset_id: chain_info.name.clone(),
        },
        location: dataset_location.to_string(),
        ..Default::default()
    };

    let retry_config = RetryConfig::default();
    retry(
        || async {
            match dataset_client.create(&metadata).await {
                Ok(_) => {
                    info!(
                        "Dataset successfully created for chain_name: {}, project_id: {}",
                        chain_info.name, project_id
                    );
                    Ok::<(), anyhow::Error>(())
                }
                Err(BigQueryError::Response(resp)) if resp.message.contains("Already Exists") => {
                    info!("Dataset '{}' already exists", chain_info.name);
                    Ok(())
                }
                Err(e) => Err(e.into()),
            }
        },
        &retry_config,
        "create_dataset",
    )
    .await?;

    Ok(())
}

// Create a table
pub async fn create_table(chain_info: &ChainInfo, table: &Table) -> Result<()> {
    let (client, project_id) = &*get_client().await?;
    let table_client = client.table(); // Create BigqueryTableClient

    // Check if table exists
    if verify_table(client, project_id, chain_info, &table.to_string()).await? {
        info!(
            "Table '{}.{}' already exists and is accessible",
            chain_info.name,
            table.to_string()
        );
        return Ok(());
    }

    let schema = match table {
        Table::Blocks => block_schema(chain_info.schema),
        Table::Logs => log_schema(chain_info.schema),
        Table::Transactions => transaction_schema(chain_info.schema),
        Table::Traces => trace_schema(chain_info.schema),
    };

    let metadata = BigQueryTable {
        table_reference: TableReference {
            project_id: project_id.clone(),
            dataset_id: chain_info.name.clone(),
            table_id: table.to_string(),
        },
        schema: Some(schema),
        time_partitioning: Some(TimePartitioning {
            partition_type: TimePartitionType::Day,
            field: Some("block_date".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let retry_config = RetryConfig::default();
    retry(
        || async {
            match table_client.create(&metadata).await {
                Ok(_) => {
                    info!(
                        "Table '{}' successfully created in dataset '{}'",
                        table.to_string(),
                        chain_info.name
                    );
                    Ok::<(), anyhow::Error>(())
                }
                Err(e) => {
                    // Check for specific "Already Exists" error using status code
                    if let BigQueryError::Response(resp) = &e {
                        if resp.message.contains("Already Exists") {
                            info!(
                                "Table '{}' already exists in dataset '{}'",
                                table.to_string(),
                                chain_info.name
                            );
                            return Ok(()); // Treat as success for the retry logic
                        }
                    }
                    Err(e.into())
                }
            }
        },
        &retry_config,
        "create_table",
    )
    .await?;

    Ok(())
}

// Insert data into a table
pub async fn insert_data<T: serde::Serialize>(
    chain_name: &str,
    table_id: &str,
    data: Vec<T>,
    block_range: (u64, u64),
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
    if let Some(metrics) = Metrics::global() {
        metrics.record_bigquery_batch_size_with_table(table_id, total_rows as f64);
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
            return Ok(false);
        }

        // Step 1: Try to generate all insert IDs. Collect results.
        let insert_ids_results: Vec<Result<String>> = batch
            .iter()
            .map(|item| generate_insert_id(table_id, item, block_range.0))
            .collect();

        // Step 2: Check if any ID generation failed. Propagate the first error found.
        let insert_ids: Vec<String> = insert_ids_results
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        // Step 3: If all IDs generated successfully, create the TableRow vector.
        let rows: Vec<TableRow<&T>> = batch
            .iter()
            .zip(insert_ids.iter()) // Zip items with their generated IDs
            .map(|(item, insert_id)| TableRow {
                insert_id: Some(insert_id.clone()), // Clone the ID string
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
                        if let Some(insert_errors) = response.insert_errors {
                            if !insert_errors.is_empty() {
                                return Err(anyhow::anyhow!("Insert errors: {:?}", insert_errors));
                            }
                        }
                        Ok(())
                    }
                    Err(e) => Err(e.into()),
                }
            },
            &retry_config,
            "insert_data",
        )
        .await?;

        batch.clear();
        Ok(true)
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
    if let Some(metrics) = Metrics::global() {
        metrics.record_bigquery_insert_latency_with_table(
            table_id,
            batch_start.elapsed().as_secs_f64(),
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
) -> Result<String> {
    let value = serde_json::to_value(data)?;

    let block_number = value
        .get("block_number")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_else(|| {
            // Log if block_number is missing for non-block tables
            if table_id != "blocks" {
                warn!(
                    "Missing block_number in data for table '{}', using fallback: {}",
                    table_id, fallback_block_number
                );
            }
            fallback_block_number
        });

    match table_id {
        "blocks" => Ok(format!("block-{}", block_number)),
        "transactions" => value.get("tx_hash").and_then(|v| v.as_str()).map_or_else(
            || {
                Err(anyhow::anyhow!(
                    "tx_hash for transactions table (block {})",
                    block_number
                ))
            },
            |tx_hash| Ok(format!("tx-{}-{}", block_number, tx_hash)),
        ),
        "logs" => {
            let tx_hash_opt = value.get("tx_hash").and_then(serde_json::Value::as_str);
            let tx_index_opt = value.get("tx_index").and_then(serde_json::Value::as_u64);
            let log_index_opt = value.get("log_index").and_then(serde_json::Value::as_u64);

            match (tx_hash_opt, tx_index_opt, log_index_opt) {
                (Some(h), Some(txi), Some(li)) => {
                    Ok(format!("log-{}-{}-{}-{}", block_number, h, txi, li))
                }
                _ => {
                    let mut missing = Vec::new();
                    if tx_hash_opt.is_none() {
                        missing.push("tx_hash");
                    }
                    if tx_index_opt.is_none() {
                        missing.push("tx_index");
                    }
                    if log_index_opt.is_none() {
                        missing.push("log_index");
                    }
                    Err(anyhow::anyhow!(
                        "[{}] for logs table (block {})",
                        missing.join(", "),
                        block_number
                    ))
                }
            }
        }
        "traces" => {
            let tx_hash_opt = value.get("tx_hash").and_then(|v| v.as_str());
            let addr_array_opt = value.get("trace_address").and_then(|v| v.as_array());

            match (tx_hash_opt, addr_array_opt) {
                (Some(h), Some(addr_array)) => {
                    let trace_address = addr_array
                        .iter()
                        .map(|v| v.as_u64().unwrap_or(0).to_string())
                        .collect::<Vec<String>>()
                        .join("-");
                    Ok(format!("trace-{}-{}-{}", block_number, h, trace_address))
                }
                _ => {
                    let mut missing = Vec::new();
                    if tx_hash_opt.is_none() {
                        missing.push("tx_hash");
                    }
                    if addr_array_opt.is_none() {
                        missing.push("trace_address");
                    }
                    Err(anyhow::anyhow!(
                        "[{}] for traces table (block {})",
                        missing.join(", "),
                        block_number
                    ))
                }
            }
        }
        _ => Err(anyhow::anyhow!(
            "Unknown table type '{}' for insertId generation (block {})",
            table_id,
            block_number
        )),
    }
}

// Get the last processed block number from storage
pub async fn get_last_processed_block(
    chain_info: &ChainInfo,
    datasets: &Vec<Table>,
) -> Result<u64> {
    let (client, project_id) = &*get_client().await?;
    let job_client = client.job(); // Create BigqueryJobClient
    let mut min_block: Option<u64> = None;

    for table in datasets {
        let table_name = table.to_string();
        // Skip tables that don't exist
        if !verify_table(client, project_id, chain_info, &table_name).await? {
            continue;
        }

        let query = format!(
            "SELECT MAX(block_number) AS max_block FROM `{project_id}.{}.{table_name}`",
            chain_info.name,
        );
        let request = QueryRequest {
            query,
            ..Default::default()
        };
        match job_client.query(project_id, &request).await {
            Ok(result) => {
                if let Some(rows) = result.rows {
                    if !rows.is_empty() {
                        // Safely access the first row and first column
                        if let Some(row) = rows.first() {
                            if let Some(cell) = row.f.first() {
                                if let Value::String(str_value) = &cell.v {
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
                }
            }
            Err(e) => {
                // If querying any table fails, propagate the error immediately.
                error!(
                    "Failed to query max block for table '{}.{}.{}': {}",
                    project_id, chain_info.name, table_name, e
                );
                return Err(e.into());
            }
        }
    }
    let min_block = min_block.unwrap_or(0);
    info!(
        "Last processed block across specified tables: {}",
        min_block
    );
    Ok(min_block)
}
