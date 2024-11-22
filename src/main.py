import asyncio
import time
from loguru import logger
import pandas as pd
import sys

from data_manager import get_data_manager
from indexer import EVMIndexer
from data_types import ChainType
from utils.utils import load_config
from utils.state_tracker import MissingBlockTracker

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

# Initialize MissingBlockTracker
missing_block_tracker = MissingBlockTracker(filepath="missing_blocks.json")

async def main_indexing_loop():
    try:
        start_time = time.time()
        logger.info("Starting indexing process")
        logger.info(f"Processing {config.chain.name} chain")
        
        # Config
        buffer = 10  # blocks
        hard_limit = 100  # blocks
        retry_interval = 10  # seconds
        batch_size = 100
        
        # Initialize
        # last_processed_block = data_manager.get_last_processed_block()
        last_processed_block = 49558000
        block_number_to_process = last_processed_block + 1 if last_processed_block > 0 else 0
        logger.info(f"Last processed block: {last_processed_block}")
        logger.info(f"Starting indexer from block {block_number_to_process}")
        last_retry_time = 0
        
        blocks_list = []
        transactions_list = []
        logs_list = []
        
        is_saving_batch = False  # Flag to track when we're saving data. This acts as a lock to prevent race conditions

        while True:
            current_time = time.time()
            
            # Only check for retries if we're not currently saving a batch
            if current_time - last_retry_time > retry_interval and not is_saving_batch:
                retry_block = missing_block_tracker.get_first_block()
                if retry_block is not None:
                    logger.info(f"Attempting to retry block {retry_block}")
                    
                    # Attempt to process the retry block
                    raw_block = await evm_indexer.get_block(retry_block)
                    if raw_block is None:
                        logger.error(f"Failed to fetch retry block {retry_block}")
                        last_retry_time = current_time
                        continue
                        
                    # Check if critical fields are still missing
                    if raw_block.get('l1_batch_number') is None or raw_block.get('l1_batch_time') is None:
                        logger.warning(f"Retry block {retry_block} still has missing data, will retry again in {retry_interval} seconds")
                    else:
                        # Data is now complete, remove from retry tracker
                        missing_block_tracker.remove_block(retry_block)
                        logger.info(f"Successfully found complete data for block {retry_block}")
                            
                last_retry_time = current_time

            # Normal block processing
            current_block_number = await evm_indexer.get_block_number()

            # If indexer gets too close to tip, back off and retry
            if block_number_to_process > (current_block_number - hard_limit - buffer):
                logger.info(f"Next block ready to process is within {current_block_number - block_number_to_process} blocks of chain tip")
                logger.info(f"Waiting for block {block_number_to_process} to be at least {hard_limit} blocks behind tip ({current_block_number})")
                
                await asyncio.sleep(1) # Changed to asyncio.sleep for non-blocking
                continue
                
            # Process next block in sequence
            raw_block = await evm_indexer.get_block(block_number_to_process)
            if raw_block is None:
                logger.error(f"Failed to fetch block {block_number_to_process}")
                block_number_to_process += 1
                continue
            
            # Check for missing data and add to retry tracker if needed
            if raw_block.get('l1BatchNumber') is None or raw_block.get('l1BatchTimestamp') is None:
                logger.warning(f"Block {block_number_to_process} has missing data, adding to retry tracker")
                missing_block_tracker.add_block(block_number_to_process)
                
            # Always process the block regardless of missing data
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

                # Add to batch lists
                blocks_list.append(block_data.block)
                if block_data.transactions:
                    transactions_list.extend(block_data.transactions)
                if block_data.logs:
                    logs_list.extend(block_data.logs)

                # When batch size is reached, save to BigQuery
                if len(blocks_list) >= batch_size:
                    batch_start = time.time()
                    is_saving_batch = True  # Set flag before saving
                    logger.info(f"is_saving_batch: {is_saving_batch}")
                    try:
                        blocks_df = pd.DataFrame([dict(block) for block in blocks_list])
                        transactions_df = pd.DataFrame([dict(tx) for tx in transactions_list]) if transactions_list else pd.DataFrame()
                        logs_df = pd.DataFrame([dict(log) for log in logs_list]) if logs_list else pd.DataFrame()

                        # Load data to BigQuery based on active datasets
                        if not blocks_df.empty and "blocks" in config.datasets:
                            data_manager.load_table(
                                df=blocks_df,
                                table_id="blocks",
                                if_exists='append'
                            )
                
                        if not transactions_df.empty and "transactions" in config.datasets:
                            data_manager.load_table(
                                df=transactions_df,
                                table_id="transactions",
                                if_exists='append'
                            )
                
                        if not logs_df.empty and "logs" in config.datasets:
                            data_manager.load_table(
                                df=logs_df,
                                table_id="logs",
                                if_exists='append'
                            )
                        batch_duration = time.time() - batch_start
                        logger.info(f"Saved batch of {batch_size} blocks to BigQuery in {batch_duration:.2f} seconds")
                    finally:
                        is_saving_batch = False  # Always reset flag after saving
                        logger.info(f"is_saving_batch: {is_saving_batch}")
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

async def main():
    """
    Main entry point for the indexer.
    """
    await asyncio.gather(
        main_indexing_loop()
    )

if __name__ == "__main__":
    try:
        # Run the main function asynchronously
        asyncio.run(main())
    except KeyboardInterrupt:
        logger.info("Program interrupted by user. Exiting.")
    except Exception as e:
        logger.exception(f"An unexpected error occurred in the main loop: {e}")
        sys.exit(1)
