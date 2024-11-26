from prometheus_client import Counter, Gauge, Histogram, Summary, start_http_server
from typing import Dict

# Block processing metrics
BLOCKS_PROCESSED = Counter(
    'indexer_blocks_processed_total',
    'Total number of blocks processed',
    ['chain']
)

BLOCK_PROCESSING_TIME = Histogram(
    'indexer_block_processing_seconds',
    'Time spent processing blocks',
    ['chain'],
    buckets=[.005, .01, .025, .05, .075, .1, .25, .5, .75, 1.0, 2.5, 5.0]
)

# Transaction metrics
TRANSACTIONS_PROCESSED = Counter(
    'indexer_transactions_processed_total',
    'Total number of transactions processed',
    ['chain']
)

TRANSACTION_VALUE = Summary(
    'indexer_transaction_value_eth',
    'Transaction values in ETH',
    ['chain']
)

# Chain metrics
CURRENT_BLOCK = Gauge(
    'indexer_current_block_number',
    'Current block number being processed',
    ['chain']
)

CHAIN_HEAD_BLOCK = Gauge(
    'indexer_chain_head_block_number',
    'Latest block number on chain',
    ['chain']
)

SYNC_LAG = Gauge(
    'indexer_sync_lag_blocks',
    'Number of blocks behind chain head',
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
    buckets=[.005, .01, .025, .05, .075, .1, .25, .5, .75, 1.0, 2.5, 5.0]
)

# Storage metrics
STORAGE_OPERATIONS = Counter(
    'indexer_storage_operations_total',
    'Total number of storage operations',
    ['chain', 'operation', 'status']
)

STORAGE_LATENCY = Histogram(
    'indexer_storage_latency_seconds',
    'Storage operation latency',
    ['chain', 'operation'],
    buckets=[.005, .01, .025, .05, .075, .1, .25, .5, .75, 1.0, 2.5, 5.0]
)

def start_metrics_server(port: int = 8000, addr: str = ''):
    """Start Prometheus metrics server
    
    Args:
        port (int): Port to listen on
        addr (str): Address to bind to (default: all interfaces)
    """
    start_http_server(port, addr)
