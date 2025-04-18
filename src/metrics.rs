use std::sync::Arc;
use tracing::info;

use axum::{routing::get, Router};
use opentelemetry::metrics::{Counter, Gauge, Histogram, MeterProvider};
use opentelemetry_sdk::metrics::{MetricError, SdkMeterProvider};
use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr;

#[derive(Clone)]
pub struct Metrics {
    registry: Arc<prometheus::Registry>,
    _provider: SdkMeterProvider,
    pub chain_name: String,

    // Block processing metrics
    pub blocks_processed: Counter<u64>,
    pub blocks_per_second: Gauge<f64>,
    pub latest_processed_block: Gauge<u64>,
    pub latest_block_processing_time: Gauge<f64>,

    // Chain metrics
    pub chain_tip_block: Gauge<u64>,
    pub chain_tip_lag: Gauge<u64>,

    // RPC metrics
    pub rpc_requests: Counter<u64>,
    pub rpc_errors: Counter<u64>,
    pub rpc_latency: Histogram<f64>,

    // BigQuery metrics
    pub bigquery_insert_latency: Histogram<f64>,
    pub bigquery_batch_size: Histogram<f64>,
}

impl Metrics {
    pub fn new(chain_name: String) -> Result<Self, MetricError> {
        // Create a new prometheus registry
        let registry = prometheus::Registry::new();

        // Configure OpenTelemetry to use this registry
        let exporter = opentelemetry_prometheus::exporter()
            .with_registry(registry.clone())
            .build()?;

        // Set up a meter to create instruments
        let provider = SdkMeterProvider::builder().with_reader(exporter).build();
        let meter = provider.meter("indexer_metrics");

        let blocks_processed = meter
            .u64_counter("indexer_blocks_processed")
            .with_description("Total number of blocks processed")
            .build();

        let blocks_per_second = meter
            .f64_gauge("indexer_blocks_per_second")
            .with_description("Average number of blocks processed per second")
            .build();

        let latest_processed_block = meter
            .u64_gauge("indexer_latest_processed_block_number")
            .with_description("Latest block number processed")
            .build();

        let latest_block_processing_time = meter
            .f64_gauge("indexer_latest_block_processing")
            .with_description("Time spent processing the latest block")
            .build();

        let chain_tip_block = meter
            .u64_gauge("indexer_chain_tip_block_number")
            .with_description("Latest block number on chain")
            .build();

        let chain_tip_lag = meter
            .u64_gauge("indexer_chain_tip_lag")
            .with_description("Number of blocks behind chain tip")
            .build();

        let rpc_requests = meter
            .u64_counter("indexer_rpc_requests")
            .with_description("Number of RPC requests made")
            .build();

        let rpc_errors = meter
            .u64_counter("indexer_rpc_errors")
            .with_description("Number of RPC errors encountered")
            .build();

        let rpc_latency = meter
            .f64_histogram("indexer_rpc_latency")
            .with_description("RPC request latency")
            .with_boundaries(vec![
                0.025, 0.05, 0.075, 0.1, 0.15, 0.2, 0.3, 0.5, 1.0, 5.0, 10.0,
            ])
            .with_unit("s")
            .build();

        let bigquery_insert_latency = meter
            .f64_histogram("indexer_bigquery_insert_latency")
            .with_description("BigQuery insert operation latency")
            .with_boundaries(vec![0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 30.0, 60.0])
            .with_unit("s")
            .build();

        let bigquery_batch_size = meter
            .f64_histogram("indexer_bigquery_batch_size")
            .with_description("Size of BigQuery insert batches")
            .with_boundaries(vec![10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0, 10000.0])
            .with_unit("rows")
            .build();

        Ok(Self {
            registry: Arc::new(registry),
            _provider: provider,
            chain_name,
            blocks_processed,
            blocks_per_second,
            latest_processed_block,
            latest_block_processing_time,
            chain_tip_block,
            chain_tip_lag,
            rpc_requests,
            rpc_errors,
            rpc_latency,
            bigquery_insert_latency,
            bigquery_batch_size,
        })
    }

    pub async fn start_metrics_server(&self, addr: &str, port: u16) {
        // Parse socket address, with error handling
        let addr = match format!("{addr}:{port}").parse::<SocketAddr>() {
            Ok(addr) => addr,
            Err(e) => {
                tracing::error!("Invalid metrics server address format: {}:{} - {}", addr, port, e);
                return;
            }
        };
        let registry = self.registry.clone();

        let app = Router::new().route("/metrics", get(move || metrics_handler(registry.clone())));

        // Determine the access URL based on the binding address. Only used for logging.
        let access_url = if addr.ip().to_string() == "0.0.0.0" {
            format!("http://localhost:{port}/metrics")
        } else {
            format!("http://{}:{port}/metrics", addr.ip())
        };

        info!(
            "Starting metrics server - binding to {} (accessible at {})",
            addr, access_url
        );

        // Handle potential binding errors gracefully
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => listener,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::AddrInUse {
                    tracing::warn!("Metrics server address {}:{} is already in use. Metrics will not be available.", addr.ip(), port);
                } else {
                    tracing::error!("Failed to bind metrics server to {}:{}: {}", addr.ip(), port, e);
                }
                return;
            }
        };

        // Spawn the server in a separate task
        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!("Metrics server error: {}", e);
            }
        });
    }
}

async fn metrics_handler(registry: Arc<prometheus::Registry>) -> String {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = vec![];
    
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        tracing::error!("Failed to encode metrics: {}", e);
        return "Error encoding metrics".to_string();
    }
    
    match String::from_utf8(buffer) {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("Invalid UTF-8 in metrics output: {}", e);
            "Error converting metrics to UTF-8".to_string()
        }
    }
}
