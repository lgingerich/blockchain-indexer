import asyncio
from dotenv import load_dotenv
import json
from loguru import logger
import os
import pandas as pd
import sys
import time

from data_manager import BigQueryManager
from indexer import EVMIndexer
from data_types import ChainType
from utils import load_config

# Load env var
load_dotenv()
CREDS_FILE_PATH = os.getenv("CREDS_FILE_PATH")
if CREDS_FILE_PATH is None:
    raise ValueError("CREDS_FILE_PATH environment variable is not set")
assert isinstance(CREDS_FILE_PATH, str)

# Load indexer config
config = load_config()
CHAIN_NAME = config.chain.name
RPC_URLS = config.chain.rpc_urls


buffer = 10 # blocks
hard_limit = 100 # blocks

async def main():
    try:
        start_time = time.time()
        logger.info("Starting indexing process")
        
        # Setup indexer and BigQuery manager
        logger.info(f"Processing {CHAIN_NAME} chain")
        chain_type = ChainType(CHAIN_NAME)
        evm_indexer = EVMIndexer(RPC_URLS, chain_type)
        bq_manager = BigQueryManager(CREDS_FILE_PATH, CHAIN_NAME)



        # block_number_to_process = 0
        block_number_to_process = 269959526

        batch_size = 100
        blocks_list = []
        transactions_list = []
        logs_list = []
        
        while True:
        # while block_number < 10:
        # while block_number < 100_001:
            loop_start = time.time()
            
            # Get current block number
            current_block_number = await evm_indexer.get_block_number()

            # If indexer gets too close to tip, back off and retry
            if block_number_to_process > (current_block_number - hard_limit - buffer):
                logger.info(f"Next block ready to process is within {current_block_number - block_number_to_process} blocks of chain tip")
                logger.info(f"Waiting for block {block_number_to_process} to be at least {hard_limit} blocks behind tip ({current_block_number})")
                
                time.sleep(1) # should configure this to be dynamic or based on chain block times
                continue

            # Get and process block data
            try:
                # Get raw block and transaction receipts
                raw_block = await evm_indexer.get_block(block_number_to_process)
                if raw_block is None:
                    logger.error(f"Failed to fetch block data for block {block_number_to_process}")
                    block_number_to_process += 1
                    continue
                # print(raw_block)

                # # Fetch receipts for all transactions
                # receipts = []
                # for tx in raw_block['transactions']:
                #     receipt = await evm_indexer.get_transaction_receipt(tx['hash'])
                #     if receipt is None:
                #         logger.error(f"Failed to fetch receipt for transaction {tx['hash']}")
                #         continue
                #     receipts.append(receipt)

                # # Parse all block data at once
                # block_data = await evm_indexer.parse_block_data(
                #     timestamp=raw_block['timestamp'],
                #     block=raw_block,
                #     receipts=receipts
                # )

                # # Add to batch lists
                # blocks_list.append(block_data.block)
                # if block_data.transactions:
                #     transactions_list.extend(block_data.transactions)
                # if block_data.logs:
                #     logs_list.extend(block_data.logs)

                # # When batch size is reached, save to BigQuery
                # if len(blocks_list) >= batch_size:
                #     batch_start = time.time()
                    
                #     blocks_df = pd.DataFrame([dict(block) for block in blocks_list])
                #     transactions_df = pd.DataFrame([dict(tx) for tx in transactions_list]) if transactions_list else pd.DataFrame()
                #     logs_df = pd.DataFrame([dict(log) for log in logs_list]) if logs_list else pd.DataFrame()

                #     # Load data to BigQuery
                #     if not blocks_df.empty:
                #         bq_manager.load_table(
                #             df=blocks_df,
                #             table_id="blocks",
                #             if_exists='append'
                #         )
                
                #     if not transactions_df.empty:
                #         bq_manager.load_table(
                #             df=transactions_df,
                #             table_id="transactions",
                #             if_exists='append'
                #         )
                
                #     if not logs_df.empty:
                #         bq_manager.load_table(
                #             df=logs_df,
                #             table_id="logs",
                #             if_exists='append'
                #         )

                #     batch_duration = time.time() - batch_start
                #     logger.info(f"Saved batch of {batch_size} blocks to BigQuery in {batch_duration:.2f} seconds")
                    
                #     # Clear the lists for next batch
                #     blocks_list = []
                #     transactions_list = []
                #     logs_list = []

                loop_duration = time.time() - loop_start
                # logger.info(f"Processed block {block_number} in {loop_duration:.2f} seconds")
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
    finally:
        total_duration = time.time() - start_time
        logger.info(f"Total execution time: {total_duration:.2f} seconds")
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
