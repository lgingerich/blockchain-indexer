use std::time::Duration;
use tracing::info;
use std::sync::Arc;

use std::net::SocketAddr;
use axum::{Router, routing::get};
use opentelemetry::{
    metrics::{MeterProvider, Counter, Histogram, Gauge, ObservableCounter},
    KeyValue
};
use opentelemetry_sdk::metrics::{MetricError, SdkMeterProvider};
use prometheus::{Encoder, TextEncoder};

pub struct Metrics {
    registry: Arc<prometheus::Registry>,
    provider: SdkMeterProvider,
    pub chain_name: String,

    // Block processing metrics
    // pub blocks_processed: ObservableCounter<u64>,
    pub blocks_processed: Counter<u64>,
    pub latest_processed_block: Gauge<u64>,
    pub latest_block_processing_time: Gauge<f64>,

    // Chain metrics
    pub chain_tip_block: Gauge<u64>,
    pub chain_tip_lag: Gauge<u64>,

    // RPC metrics
    pub rpc_requests: Counter<u64>,
    pub rpc_errors: Counter<u64>,
    pub rpc_latency: Histogram<f64>,
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
            .u64_counter("indexer_blocks_processed_total")
            .with_description("Total number of blocks processed")
            .build();

        let latest_processed_block = meter
            .u64_gauge("indexer_latest_processed_block_number")
            .with_description("Latest block number processed")
            .build();

        let latest_block_processing_time = meter
            .f64_gauge("indexer_latest_block_processing_seconds")
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
            .u64_counter("indexer_rpc_requests_total")
            .with_description("Total number of RPC requests made")
            .build();

        let rpc_errors = meter
            .u64_counter("indexer_rpc_errors_total")
            .with_description("Total number of RPC errors encountered")
            .build();

        let rpc_latency = meter
            .f64_histogram("indexer_rpc_latency_seconds")
            .with_description("RPC request latency")
            .with_boundaries(vec![0.025, 0.05, 0.075, 0.1, 0.15, 0.2, 0.3, 0.5, 1.0, 5.0, 10.0])
            .with_unit("s")
            .build();

        Ok(Self {
            registry: Arc::new(registry),
            provider,
            chain_name,
            blocks_processed,
            latest_processed_block,
            latest_block_processing_time,
            chain_tip_block,
            chain_tip_lag,
            rpc_requests,
            rpc_errors,
            rpc_latency,
        })
    }

    pub async fn start_metrics_server(&self, addr: &str, port: u16) {
        let addr = format!("{}:{}", addr, port).parse::<SocketAddr>().unwrap();
        let registry = self.registry.clone();
        
        let app = Router::new()
            .route("/metrics", get(move || metrics_handler(registry.clone())));

        // Determine the access URL based on the binding address. Only used for logging.

        let access_url = if addr.ip().to_string() == "0.0.0.0" {
            format!("http://localhost:{}/metrics", port)
        } else {
            format!("http://{}:{}/metrics", addr.ip(), port)
        };

        info!("Starting metrics server - binding to {} (accessible at {})", 
            addr, 
            access_url
        );

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .unwrap();
        
        // Spawn the server in a separate task
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
    }
}

async fn metrics_handler(registry: Arc<prometheus::Registry>) -> String {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}