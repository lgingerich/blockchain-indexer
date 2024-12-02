from prometheus_client import Counter, Gauge, Histogram, start_http_server
from loguru import logger

# Block processing metrics
BLOCKS_PROCESSED = Counter(
    'indexer_blocks_processed_total',
    'Total number of blocks processed',
    ['chain']
)

LATEST_PROCESSED_BLOCK = Gauge(
    'indexer_latest_processed_block_number',
    'Latest block number processed',
    ['chain']
)

LATEST_BLOCK_PROCESSING_TIME = Gauge(
    'indexer_latest_block_processing_seconds',
    'Time spent processing the latest block',
    ['chain']
)

# Chain metrics
CHAIN_TIP_BLOCK = Gauge(
    'indexer_chain_tip_block_number',
    'Latest block number on chain',
    ['chain']
)

CHAIN_TIP_LAG = Gauge(
    'indexer_chain_tip_lag',
    'Number of blocks behind chain tip',
    ['chain']
)

# RPC metrics
RPC_REQUESTS = Counter(
    'indexer_rpc_requests_total',
    'Total number of RPC requests made',
    ['chain', 'method']
)

RPC_ERRORS = Counter(
    'indexer_rpc_errors_total',
    'Total number of RPC errors encountered',
    ['chain', 'method']
)

RPC_LATENCY = Histogram(
    'indexer_rpc_latency_seconds',
    'RPC request latency',
    ['chain', 'method'],
    buckets=[0.025, 0.05, 0.075, 0.1, 0.15, 0.2, 0.3, 0.5, 1.0, 5.0, 10.0]
)

def start_metrics_server(port: int = 9100, addr: str = '0.0.0.1'):
    """Start Prometheus metrics server
    
    Args:
        port (int): Port to listen on
        addr (str): Address to bind to (default: all interfaces)
    """
    # Start the metrics HTTP server
    logger.info(f"Starting Prometheus metrics server on {addr}:{port}")
    start_http_server(port, addr)
    
    # Reset all metrics to clear any stale values
    metrics_to_clear = [
        BLOCKS_PROCESSED,
        LATEST_PROCESSED_BLOCK, 
        LATEST_BLOCK_PROCESSING_TIME,
        CHAIN_TIP_BLOCK,
        CHAIN_TIP_LAG,
        RPC_REQUESTS,
        RPC_ERRORS,
        RPC_LATENCY
    ]
    
    for metric in metrics_to_clear:
        metric._metrics.clear()
        
    logger.info("All metrics initialized to clean state")
