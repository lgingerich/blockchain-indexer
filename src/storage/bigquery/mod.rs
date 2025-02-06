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
        Ok(_) => Ok(client_arc),
        Err(_) => {
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
        &RetryConfig::default(),
        &format!("create_dataset_{}", chain_name),
    )
    .await
}

// Create a table
pub async fn create_table(chain_name: &str, table_id: &str, chain: Chain) -> Result<()> {
    let (client, project_id) = &*get_client().await?;
    let table_client = client.table(); // Create BigqueryTableClient

    // Check if table exists
    if verify_table(client, project_id, chain_name, table_id).await? {
        info!("Table '{}.{}' already exists and is accessible", chain_name, table_id);
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
        ..Default::default()
    };

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
        &RetryConfig::default(),
        &format!("create_table_{}_{}", chain_name, table_id),
    )
    .await
}

// Insert data into a table
pub async fn insert_data<T: serde::Serialize>(
    chain_name: &str,
    table_id: &str,
    data: Vec<T>,
    block_number: u64,
) -> Result<()> {
    let (client, project_id) = &*get_client().await?;
    let tabledata_client = client.tabledata();

    if data.is_empty() {
        info!(
            "No data to insert into {}.{}.{} for block {}",
            project_id, chain_name, table_id, block_number
        );
        return Ok(());
    }

    const BATCH_SIZE: usize = 100_000;
    let total_rows = data.len();

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

        retry(
            || async {
                match tabledata_client
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
                        Err(anyhow!("Data insertion failed"))
                    }
                }
            },
            &RetryConfig::default(),
            &format!("insert_data_{}_{}_{}", chain_name, table_id, block_number),
        )
        .await?;
    }

    info!(
        "Successfully inserted {} rows into {}.{}.{} for block {}",
        total_rows, project_id, chain_name, table_id, block_number
    );

    Ok(())
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
            "SELECT MAX(block_number) AS max_block FROM `{}.{}.{}`",
            project_id, chain_name, table_id
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
