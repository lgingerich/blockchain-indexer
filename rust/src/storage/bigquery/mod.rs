mod schema;

use anyhow::{anyhow, Result};
use google_cloud_bigquery::client::{Client, ClientConfig};
use google_cloud_bigquery::http::dataset::{Dataset, DatasetReference};
use google_cloud_bigquery::http::error::Error as BigQueryError;
use google_cloud_bigquery::http::job::query::QueryRequest;
use google_cloud_bigquery::http::table::{Table, TableReference};
use google_cloud_bigquery::http::tabledata::{
    insert_all::{InsertAllRequest, Row as TableRow},
    list::Value,
};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tracing::{error, info};

use crate::models::common::Chain;
use crate::storage::bigquery::schema::{
    block_schema, log_schema, trace_schema, transaction_schema,
};
use crate::utils::retry::{retry, RetryConfig};

// Define a static OnceCell to hold the shared Client and Project ID
static BIGQUERY_CLIENT: OnceCell<Arc<(Client, String)>> = OnceCell::new();

// Initializes and returns the shared BigQuery Client and Project ID.
// This function ensures that the Client is initialized only once.
async fn get_client() -> Result<Arc<(Client, String)>> {
    if let Some(client) = BIGQUERY_CLIENT.get() {
        return Ok(client.clone());
    }

    let (config, project_id_option) = ClientConfig::new_with_auth().await?;
    let client = Client::new(config).await?;
    let project_id = project_id_option.ok_or_else(|| anyhow!("Project ID not found"))?;

    let client_arc = Arc::new((client, project_id));

    match BIGQUERY_CLIENT.set(client_arc.clone()) {
        Ok(_) => Ok(client_arc),
        Err(_) => {
            // If we failed to set (because another thread set it first),
            // return the value that was set by the other thread
            Ok(BIGQUERY_CLIENT.get().unwrap().clone())
        }
    }
}

// Verify that a dataset exists and is accessible
async fn verify_dataset(client: &Client, project_id: &str, dataset_id: &str) -> Result<bool> {
    match client.dataset().get(project_id, dataset_id).await {
        Ok(_) => Ok(true),
        Err(BigQueryError::Response(resp)) if resp.message.contains("Not found") => Ok(false),
        Err(e) => Err(anyhow!("Failed to verify dataset: {}", e)),
    }
}

// Verify that a table exists and is accessible
async fn verify_table(
    client: &Client,
    project_id: &str,
    dataset_id: &str,
    table_id: &str,
) -> Result<bool> {
    match client.table().get(project_id, dataset_id, table_id).await {
        Ok(_) => Ok(true),
        Err(BigQueryError::Response(resp)) if resp.message.contains("Not found") => Ok(false),
        Err(e) => Err(anyhow!("Failed to verify table: {}", e)),
    }
}

// Create a dataset
async fn create_dataset(dataset_id: &str) -> Result<()> {
    let (client, project_id) = &*get_client().await?;
    let dataset_client = client.dataset();

    let metadata = Dataset {
        dataset_reference: DatasetReference {
            project_id: project_id.clone(),
            dataset_id: dataset_id.to_string(),
        },
        ..Default::default()
    };

    match dataset_client.create(&metadata).await {
        Ok(_) => {
            info!(dataset_id, project_id = ?project_id, "Dataset successfully created");
            Ok(())
        }
        Err(e) => {
            match e {
                BigQueryError::Response(resp) => {
                    if resp.message.contains("Already Exists") {
                        info!("Dataset '{}' already exists", dataset_id);
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
            Err(anyhow!("Dataset creation failed"))
        }
    }
}

pub async fn create_dataset_with_retry(dataset_id: &str) -> Result<()> {
    let (client, project_id) = &*get_client().await?;
    let retry_config = RetryConfig::default();

    retry(
        || async {
            // Check if dataset exists
            if verify_dataset(client, project_id, dataset_id).await? {
                info!("Dataset '{}' already exists and is accessible", dataset_id);
                return Ok(());
            }

            // Create and verify dataset
            create_dataset(dataset_id).await?;
            if verify_dataset(client, project_id, dataset_id).await? {
                info!("Dataset '{}' created and verified", dataset_id);
                Ok(())
            } else {
                Err(anyhow!("Dataset creation could not be verified"))
            }
        },
        &retry_config,
        &format!("create_dataset_{}", dataset_id),
    )
    .await
}

async fn create_table(dataset_id: &str, table_id: &str, chain: Chain) -> Result<()> {
    let (client, project_id) = &*get_client().await?;
    let table_client = client.table(); // Create BigqueryTableClient
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
            dataset_id: dataset_id.to_string(),
            table_id: table_id.to_string(),
        },
        schema: Some(schema),
        ..Default::default()
    };

    match table_client.create(&metadata).await {
        Ok(_) => {
            info!(
                "Table '{}' successfully created in dataset '{}'",
                table_id, dataset_id
            );
            Ok(())
        }
        Err(e) => {
            match e {
                BigQueryError::Response(resp) => {
                    if resp.message.contains("Already Exists") {
                        info!(
                            "Table '{}' already exists in dataset '{}'",
                            table_id, dataset_id
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
}

pub async fn create_table_with_retry(dataset_id: &str, table_id: &str, chain: Chain) -> Result<()> {
    let (client, project_id) = &*get_client().await?;
    let retry_config = RetryConfig::default();

    retry(
        || async {
            // Check if table exists
            if verify_table(client, project_id, dataset_id, table_id).await? {
                info!(
                    "Table '{}.{}' already exists and is accessible",
                    dataset_id, table_id
                );
                return Ok(());
            }

            // Create and verify table
            create_table(dataset_id, table_id, chain).await?;
            if verify_table(client, project_id, dataset_id, table_id).await? {
                info!("Table '{}.{}' created and verified", dataset_id, table_id);
                Ok(())
            } else {
                Err(anyhow!("Table creation could not be verified"))
            }
        },
        &retry_config,
        &format!("create_table_{}_{}", dataset_id, table_id),
    )
    .await
}

async fn insert_data<T: serde::Serialize>(
    dataset_id: &str,
    table_id: &str,
    data: &[T],
) -> Result<()> {
    let (client, project_id) = &*get_client().await?;
    let tabledata_client = client.tabledata();

    if data.is_empty() {
        info!(
            "No data to insert into {}.{}.{}",
            project_id, dataset_id, table_id
        );
        return Ok(());
    }

    // Process data in chunks of 1000 rows (you can adjust this value)
    const BATCH_SIZE: usize = 1000;
    for chunk in data.chunks(BATCH_SIZE) {
        let rows = chunk
            .iter()
            .map(|item| TableRow {
                insert_id: None,
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

        match tabledata_client
            .insert(project_id, dataset_id, table_id, &request)
            .await
        {
            Ok(response) => {
                if let Some(errors) = response.insert_errors {
                    if !errors.is_empty() {
                        for error in errors {
                            error!(
                                "Row {} failed to insert with {} errors:",
                                error.index,
                                error.errors.len()
                            );
                            for err_msg in error.errors {
                                error!(
                                    "Reason: {}, Location: {}, Message: {}, Debug Info: {}",
                                    err_msg.reason,
                                    err_msg.location,
                                    err_msg.message,
                                    err_msg.debug_info
                                );
                            }
                        }
                        return Err(anyhow!("Some rows failed to insert"));
                    }
                }
                info!(
                    "Successfully inserted batch of {} rows into {}.{}.{}",
                    chunk.len(),
                    project_id,
                    dataset_id,
                    table_id
                );
            }
            Err(e) => {
                match e {
                    BigQueryError::Response(resp) => {
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
                return Err(anyhow!("Data insertion failed"));
            }
        }
    }

    Ok(())
}

pub async fn insert_data_with_retry<T: serde::Serialize>(
    dataset_id: &str,
    table_id: &str,
    data: Vec<T>,
) -> Result<()> {
    let (client, project_id) = &*get_client().await?;
    let retry_config = RetryConfig::default();

    retry(
        || async {
            // Verify table exists before attempting insert
            if !verify_table(client, project_id, dataset_id, table_id).await? {
                return Err(anyhow!("Table not found before insert attempt"));
            }

            insert_data(dataset_id, table_id, &data).await
        },
        &retry_config,
        &format!("insert_data_{}_{}", dataset_id, table_id),
    )
    .await
}

pub async fn get_last_processed_block(dataset_id: &str, datasets: &Vec<String>) -> Result<u64> {
    let (client, project_id) = &*get_client().await?;
    let job_client = client.job(); // Create BigqueryJobClient
    let mut min_block: Option<u64> = None;

    for table_id in datasets {
        // Skip tables that don't exist
        if !verify_table(client, project_id, dataset_id, table_id).await? {
            continue;
        }

        let query = format!(
            "SELECT MAX(block_number) AS max_block FROM `{}.{}.{}`",
            project_id, dataset_id, table_id
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
