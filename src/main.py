import asyncio
import time
from loguru import logger
import pandas as pd
import sys

from data_manager import get_data_manager
from indexer import EVMIndexer
from data_types import ChainType
from utils import load_config
from metrics import (
    start_metrics_server,
    BLOCKS_PROCESSED,
    BLOCK_PROCESSING_TIME,
    CURRENT_BLOCK,
    CHAIN_HEAD_BLOCK,
    RPC_REQUESTS,
    RPC_ERRORS,
    RPC_LATENCY,
    SYNC_LAG,
    STORAGE_OPERATIONS,
    STORAGE_LATENCY
)

# Save logs to file
logger.add("logs/indexer.log", rotation="100 MB", retention="10 days")

# Load indexer config
config = load_config("config.yml")

# Initialize core components
chain_type = ChainType(config.chain.name)
evm_indexer = EVMIndexer(config.chain.rpc_urls, chain_type)
storage_config = config.storage
data_manager = get_data_manager(
    storage_type=storage_config.type,
    chain_name=config.chain.name,
    config=storage_config,
    active_datasets=config.datasets
)

async def main():
    # Start metrics server
    start_metrics_server(8000, addr='0.0.0.0')
    
    # Initialize metrics with the chain label
    chain_name = config.chain.name

    # Initialize all storage operation metrics
    for operation in ['save_blocks', 'save_transactions', 'save_logs']:
        for status in ['success', 'error']:
            STORAGE_OPERATIONS.labels(chain=chain_name, operation=operation, status=status).inc(0)
        STORAGE_LATENCY.labels(chain=chain_name, operation=operation).observe(0)

    # Initialize RPC metrics
    for method in ['get_block', 'get_block_number', 'get_transaction_receipt']:
        RPC_REQUESTS.labels(chain=chain_name, method=method).inc(0)
        RPC_ERRORS.labels(chain=chain_name, method=method).inc(0)
        RPC_LATENCY.labels(chain=chain_name, method=method).observe(0)

    # Initialize other metrics
    BLOCKS_PROCESSED.labels(chain=chain_name).inc(0)
    BLOCK_PROCESSING_TIME.labels(chain=chain_name).observe(0)
    CURRENT_BLOCK.labels(chain=chain_name).set(0)
    CHAIN_HEAD_BLOCK.labels(chain=chain_name).set(0)
    SYNC_LAG.labels(chain=chain_name).set(0)

    logger.info(f"Initialized metrics for chain: {chain_name}")
    
    try:
        logger.info("Starting indexing process")
        logger.info(f"Processing {config.chain.name} chain")
        
        # Config
        buffer = 10  # blocks
        hard_limit = 100  # blocks
        batch_size = 100
        
        # Initialize
        last_processed_block = data_manager.get_last_processed_block()
        block_number_to_process = last_processed_block + 1 if last_processed_block > 0 else 0
        logger.info(f"Last processed block: {last_processed_block}")
        logger.info(f"Starting indexer from block {block_number_to_process}")
        
        blocks_list = []
        transactions_list = []
        logs_list = []
        
        is_saving_batch = False  # Flag to track when we're saving data. This acts as a lock to prevent race conditions

        while True:           
            # Normal block processing
            current_block_number = await evm_indexer.get_block_number()
            CHAIN_HEAD_BLOCK.labels(chain=config.chain.name).set(current_block_number)
            
            # If indexer gets too close to tip, back off and retry
            if block_number_to_process > (current_block_number - hard_limit - buffer):
                logger.info(f"Next block ready to process is within {current_block_number - block_number_to_process} blocks of chain tip")
                logger.info(f"Waiting for block {block_number_to_process} to be at least {hard_limit} blocks behind tip ({current_block_number})")
                await asyncio.sleep(1)
                continue
                
            # Process next block in sequence
            raw_block = await evm_indexer.get_block(block_number_to_process)
            if raw_block is None:
                logger.error(f"Failed to fetch block {block_number_to_process}")
                block_number_to_process += 1
                continue
            
            # Check for required fields - block here until they're available
            # The blocking fields only apply for L2s sending data to L1
            # These fields are different for each L2
            if chain_type == ChainType.ZKSYNC and (raw_block.get('l1BatchNumber') is None or raw_block.get('l1BatchTimestamp') is None):
                logger.warning(f"Block {block_number_to_process} has missing required data, waiting...")
                await asyncio.sleep(1)
                continue
            elif chain_type == ChainType.ARBITRUM and raw_block.get('l1BlockNumber') is None:
                logger.warning(f"Block {block_number_to_process} has missing required data, waiting...")
                await asyncio.sleep(1)
                continue

            try:
                # Process block data (receipts, etc.)
                receipts = []
                for tx in raw_block['transactions']:
                    receipt = await evm_indexer.get_transaction_receipt(tx['hash'])
                    if receipt is None:
                        logger.error(f"Failed to fetch receipt for transaction {tx['hash']}")
                        continue
                    receipts.append(receipt)

                # Parse all block data at once
                block_data = await evm_indexer.parse_block_data(
                    timestamp=raw_block['timestamp'],
                    block=raw_block,
                    receipts=receipts
                )

                # Increment blocks processed counter
                BLOCKS_PROCESSED.labels(chain=config.chain.name).inc()

                # Add to batch lists
                blocks_list.append(block_data.block)
                if block_data.transactions:
                    transactions_list.extend(block_data.transactions)
                if block_data.logs:
                    logs_list.extend(block_data.logs)

                # Update current block and sync lag metrics
                CURRENT_BLOCK.labels(chain=config.chain.name).set(block_number_to_process)
                SYNC_LAG.labels(chain=config.chain.name).set(current_block_number - block_number_to_process)
                
                # When batch size is reached, save to storage
                if len(blocks_list) >= batch_size:
                    batch_start = time.time()
                    is_saving_batch = True

                    try:
                        blocks_df = pd.DataFrame([dict(block) for block in blocks_list])
                        transactions_df = pd.DataFrame([dict(tx) for tx in transactions_list]) if transactions_list else pd.DataFrame()
                        logs_df = pd.DataFrame([dict(log) for log in logs_list]) if logs_list else pd.DataFrame()

                        # Calculate block range for this batch
                        start_block = blocks_df['block_number'].min()
                        end_block = blocks_df['block_number'].max()

                        # Save data based on active datasets
                        df_mapping = {
                            "blocks": blocks_df,
                            "transactions": transactions_df,
                            "logs": logs_df
                        }
                        
                        for table_id, df in df_mapping.items():
                            if not df.empty and table_id in config.datasets:
                                storage_start = time.time()
                                try:
                                    data_manager.load_table(
                                        df=df,
                                        table_id=table_id,
                                        if_exists='append',
                                        start_block=start_block,
                                        end_block=end_block
                                    )
                                    STORAGE_OPERATIONS.labels(
                                        chain=config.chain.name,
                                        operation=f'save_{table_id}',
                                        status='success'
                                    ).inc()
                                except Exception as e:
                                    STORAGE_OPERATIONS.labels(
                                        chain=config.chain.name,
                                        operation=f'save_{table_id}',
                                        status='error'
                                    ).inc()
                                    raise
                                finally:
                                    STORAGE_LATENCY.labels(
                                        chain=config.chain.name,
                                        operation=f'save_{table_id}'
                                    ).observe(time.time() - storage_start)
                        batch_duration = time.time() - batch_start
                        logger.info(f"Saved batch from block {start_block} to {end_block} in {batch_duration:.2f} seconds")
                    finally:
                        is_saving_batch = False  # Always reset flag after saving

                    blocks_list = []
                    transactions_list = []
                    logs_list = []

                block_number_to_process += 1

            except Exception as e:
                logger.error(f"Error processing block {block_number_to_process}: {e}")
                block_number_to_process += 1
                continue

    except KeyError as e:
        logger.error(f"Configuration error: Missing key {e}")
        sys.exit(1)
    except ValueError as e:
        logger.error(f"Invalid configuration value: {e}")
        sys.exit(1)
    except Exception as e:
        logger.exception(f"An unexpected error occurred: {e}")
        sys.exit(1)

if __name__ == "__main__":
    try:
        # Run the main function asynchronously
        asyncio.run(main())
    except KeyboardInterrupt:
        logger.info("Program interrupted by user. Exiting.")
    except Exception as e:
        logger.exception(f"An unexpected error occurred in the main loop: {e}")
        sys.exit(1)
