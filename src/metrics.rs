use anyhow::{Context, Result};
use axum::{extract::State, http::StatusCode, routing::get, Router};
use opentelemetry::metrics::{Counter, Gauge, Histogram, MeterProvider};
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use once_cell::sync::OnceCell;

// Global metrics instance
static GLOBAL_METRICS: OnceCell<Arc<Metrics>> = OnceCell::new();

#[derive(Clone)]
pub struct Metrics {
    registry: Arc<prometheus::Registry>,
    _provider: SdkMeterProvider,

    // Pre-computed common attributes to avoid repeated allocations
    common_attributes: Vec<KeyValue>,

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
    pub fn new(chain_name: String) -> Result<Self> {
        // Create a new prometheus registry
        let registry = prometheus::Registry::new();

        // Configure OpenTelemetry to use this registry
        let exporter = opentelemetry_prometheus::exporter()
            .with_registry(registry.clone())
            .build()?;

        // Set up a meter to create instruments
        let provider = SdkMeterProvider::builder().with_reader(exporter).build();
        let meter = provider.meter("indexer_metrics");

        // Pre-compute common attributes
        let common_attributes = vec![KeyValue::new("chain", chain_name.clone())];

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
            common_attributes,
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

    // Initialize global metrics instance
    pub fn init_global(chain_name: String) -> Result<()> {
        let metrics = Self::new(chain_name)?;
        GLOBAL_METRICS.set(Arc::new(metrics))
            .map_err(|_| anyhow::anyhow!("Global metrics already initialized"))?;
        Ok(())
    }

    // Get global metrics instance
    pub fn global() -> Option<Arc<Self>> {
        GLOBAL_METRICS.get().cloned()
    }

    // Convenience methods for recording metrics with pre-computed attributes

    pub fn record_blocks_processed(&self, count: u64) {
        self.blocks_processed.add(count, &self.common_attributes);
    }

    pub fn record_blocks_per_second(&self, rate: f64) {
        self.blocks_per_second.record(rate, &self.common_attributes);
    }

    pub fn record_latest_processed_block(&self, block_num: u64) {
        self.latest_processed_block
            .record(block_num, &self.common_attributes);
    }

    pub fn record_latest_block_processing_time(&self, time_secs: f64) {
        self.latest_block_processing_time
            .record(time_secs, &self.common_attributes);
    }

    pub fn record_chain_tip(&self, block_num: u64) {
        self.chain_tip_block
            .record(block_num, &self.common_attributes);
    }

    pub fn record_chain_tip_lag(&self, lag: u64) {
        self.chain_tip_lag.record(lag, &self.common_attributes);
    }

    pub fn record_rpc_request(&self, method: &str) {
        let mut attrs = self.common_attributes.clone();
        attrs.push(KeyValue::new("method", method.to_string()));
        self.rpc_requests.add(1, &attrs);
    }

    pub fn record_rpc_error(&self, method: &str) {
        let mut attrs = self.common_attributes.clone();
        attrs.push(KeyValue::new("method", method.to_string()));
        self.rpc_errors.add(1, &attrs);
    }

    pub fn record_rpc_latency(&self, method: &str, latency_secs: f64) {
        let mut attrs = self.common_attributes.clone();
        attrs.push(KeyValue::new("method", method.to_string()));
        self.rpc_latency.record(latency_secs, &attrs);
    }

    // Optimized methods for BigQuery that include table name
    pub fn record_bigquery_insert_latency_with_table(&self, table: &str, latency_secs: f64) {
        let mut attrs = self.common_attributes.clone();
        attrs.push(KeyValue::new("table", table.to_string()));
        self.bigquery_insert_latency.record(latency_secs, &attrs);
    }

    pub fn record_bigquery_batch_size_with_table(&self, table: &str, size: f64) {
        let mut attrs = self.common_attributes.clone();
        attrs.push(KeyValue::new("table", table.to_string()));
        self.bigquery_batch_size.record(size, &attrs);
    }

    pub async fn start_metrics_server(&self, addr: &str, port: u16) -> Result<()> {
        // Parse socket address
        let addr: SocketAddr = format!("{addr}:{port}")
            .parse()
            .with_context(|| format!("Invalid metrics server address format: {addr}:{port}"))?;

        let registry = self.registry.clone();

        let app = Router::new()
            .route("/metrics", get(metrics_handler))
            .with_state(registry.clone());

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

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .with_context(|| format!("Failed to bind metrics server to {addr}"))?;

        tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .await
                .context("Failed to serve metrics");
        });

        Ok(())
    }
}



#[axum::debug_handler]
async fn metrics_handler(
    State(registry): State<Arc<prometheus::Registry>>,
) -> Result<String, (StatusCode, String)> {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = vec![];

    encoder.encode(&metric_families, &mut buffer).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to encode metrics: {}", e),
        )
    })?;

    String::from_utf8(buffer).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to convert metrics buffer to string: {}", e),
        )
    })
}
