mod schema;

use google_cloud_bigquery::client::{ClientConfig, Client};
use google_cloud_bigquery::http::dataset::{Dataset, DatasetReference};
use google_cloud_bigquery::http::table::{Table, TableFieldSchema, TableReference, TableSchema};
use google_cloud_bigquery::http::error::Error as BigQueryError;
use google_cloud_bigquery::http::tabledata::insert_all::{InsertAllRequest, Row};

use eyre::{Result, Report};
use serde::Serialize;
use tracing::{info, warn, error};
use once_cell::sync::OnceCell;

use crate::storage::bigquery::schema::{block_schema, log_schema, transaction_schema, trace_schema};

use crate::models::indexed::blocks::TransformedBlockData;
use crate::models::indexed::logs::TransformedLogData;
use crate::models::indexed::traces::TransformedTraceData;
use crate::models::indexed::transactions::TransformedTransactionData;

use std::sync::Arc;

// Define a static OnceCell to hold the shared Client and Project ID
static BIGQUERY_CLIENT: OnceCell<Arc<(Client, String)>> = OnceCell::new();

/// Initializes and returns the shared BigQuery Client and Project ID.
/// This function ensures that the Client is initialized only once.
async fn get_client() -> Result<Arc<(Client, String)>, Report> {
    if let Some(client) = BIGQUERY_CLIENT.get() {
        return Ok(client.clone());
    }

    let (config, project_id_option) = ClientConfig::new_with_auth().await?;
    let client = Client::new(config).await?;
    let project_id = project_id_option
        .ok_or_else(|| eyre::eyre!("Project ID not found"))?;
    
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

pub async fn create_dataset(dataset_id: &str) -> Result<(), Report> {
    let (client, project_id) = &*get_client().await?;
    let dataset_client = client.dataset(); // Create BigqueryDatasetClient

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
            Err(eyre::eyre!("Dataset creation failed"))
        }
    }
}

pub async fn create_table(dataset_id: &str, table_id: &str) -> Result<(), Report> {
    let (client, project_id) = &*get_client().await?;
    let table_client = client.table(); // Create BigqueryTableClient
    let schema = match table_id {
        "blocks" => block_schema(),
        "logs" => log_schema(),
        "transactions" => transaction_schema(),
        "traces" => trace_schema(),
        _ => return Err(eyre::eyre!("Invalid table ID: {}", table_id)),
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
            info!("Table '{}' successfully created in dataset '{}'", table_id, dataset_id);
            Ok(())
        }
        Err(e) => {
            match e {
                BigQueryError::Response(resp) => {
                    if resp.message.contains("Already Exists") {
                        info!("Table '{}' already exists in dataset '{}'", table_id, dataset_id);
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
            Err(eyre::eyre!("Table creation failed"))
        }
    }
}

// Sometimes get this error:
// BigQuery API Error: Table 871411803528:test_dataset.blocks not found.
pub async fn insert_data<T: serde::Serialize>(
    dataset_id: &str, 
    table_id: &str, 
    data: Vec<T>
) -> Result<(), Report> {
    let (client, project_id) = &*get_client().await?;
    let tabledata_client = client.tabledata(); // Create BigqueryTabledataClient

    if data.is_empty() {
        info!("No data to insert into {}.{}.{}", project_id, dataset_id, table_id);
        return Ok(());
    }
    
    let rows = data.into_iter()
        .map(|item| Row {
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

    match tabledata_client.insert(&project_id, dataset_id, table_id, &request).await {
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
                    Err(eyre::eyre!("Some rows failed to insert"))
                } else {
                    info!(
                        "Successfully inserted all data into {}.{}.{}",
                        project_id, dataset_id, table_id
                    );
                    Ok(())
                }
            } else {
                info!(
                    "Successfully inserted all data into {}.{}.{}",
                    project_id, dataset_id, table_id
                );
                Ok(())
            }
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
            Err(eyre::eyre!("Data insertion failed"))
        }
    }
}