# import json
# from .repository import BlockRepository, TransactionRepository, LogRepository
# from .database import get_db
# from sqlalchemy.exc import SQLAlchemyError

# async def process_block(block_data_json):
#     """
#     Process the block data received from the queue.
#     """
#     try:
#         # Parse the block data from JSON
#         block_data = json.loads(block_data_json)

#         # Extract block, transaction, and log data
#         block_info = extract_block_info(block_data)
#         transactions = block_data.get('transactions', [])

#         # Process and store block data
#         with get_db() as db:
#             block = BlockRepository.create_block(db, block_info)
#             for transaction_data in transactions:
#                 # Extract transaction and log data
#                 transaction_info = extract_transaction_info(transaction_data)
#                 logs = transaction_data.get('logs', [])

#                 # Create and store the transaction
#                 transaction = TransactionRepository.create_transaction(db, transaction_info)

#                 # Process each log
#                 for log_data in logs:
#                     log_info = extract_log_info(log_data)
#                     LogRepository.create_log(db, log_info)

#             # Commit the transaction
#             db.commit()

#     except SQLAlchemyError as e:
#         # Handle database errors
#         print(f"Database error occurred: {e}")
#         raise
#     except json.JSONDecodeError:
#         # Handle JSON parsing error
#         print("Failed to decode JSON data")
#         raise
#     except Exception as e:
#         # Handle any other exceptions
#         print(f"An error occurred: {e}")
#         raise

# def extract_block_info(block_data):
#     """
#     Extract and return block information from the block data.
#     """
#     # Extract necessary fields from block_data
#     # This is a simplified example. You need to extract all the required fields.
#     block_info = {
#         'number': block_data.get('number'),
#         'hash': block_data.get('hash'),
#         'parent_hash': block_data.get('parentHash'),
#         # Add other necessary fields...
#     }
#     return block_info

# def extract_transaction_info(transaction_data):
#     """
#     Extract and return transaction information from the transaction data.
#     """
#     # Extract necessary fields from transaction_data
#     transaction_info = {
#         'hash': transaction_data.get('hash'),
#         'block_number': transaction_data.get('blockNumber'),
#         # Add other necessary fields...
#     }
#     return transaction_info

# def extract_log_info(log_data):
#     """
#     Extract and return log information from the log data.
#     """
#     # Extract necessary fields from log_data
#     log_info = {
#         'log_index': log_data.get('logIndex'),
#         'transaction_hash': log_data.get('transactionHash'),
#         # Add other necessary fields...
#     }
#     return log_info




########################################################################################

# import asyncio
from consumer import consume_blocks
from database import get_db
from db.repository import BlockRepository
import logging
import datetime

from web3 import Web3
# import pandas as pd
import polars as pl


logger = logging.getLogger(__name__)

# async def determine_next_block_to_process():
#     with get_db() as db:
        # latest_block_in_db = BlockRepository.get_latest_block(db)
        # latest_block_in_chain = consume_blocks()
        # if latest_block_in_db:
        #     latest_block_number_in_db = latest_block_in_db.number
        #     # Now, use latest_block_number for your logic
        #     logger.info(f'latest_block_in_db = {latest_block_in_db}')
        #     logger.info(f'latest_block_in_chain = {latest_block_in_chain}')
#         else:
#             # Handle the case when there are no blocks in the database
#             print('no blocks in database')

# async def determine_next_block_to_process():
#     db = next(get_db())
#     try:
#         latest_block_in_db = BlockRepository.get_latest_block(db)
#         latest_block_in_chain = consume_blocks()
#         if latest_block_in_db:
#             latest_block_number_in_db = latest_block_in_db.number
#             # Now, use latest_block_number for your logic
#             logger.info(f'latest_block_in_db = {latest_block_in_db}')
#             logger.info(f'latest_block_in_chain = {latest_block_in_chain}')
#     finally:
#         db.close()

async def process_data():
    # await determine_next_block_to_process()

    # Connect to Ethereum node
    w3 = Web3(Web3.HTTPProvider('https://ethereum.publicnode.com'))

    # Function to get block details
    def get_block(block_number):
        block = w3.eth.get_block(block_number, full_transactions=False)
        return {
            'hash': block.hash.hex(),
            'miner': block.miner,
            'nonce': block.nonce.hex(),
            'parent_hash': block.parentHash.hex(),
            'number': block.number,
            'size': block.size,
            'time': block.timestamp,
            'total_difficulty': block.totalDifficulty,
            'base_fee_per_gas': block.baseFeePerGas if 'baseFeePerGas' in block else None,
            'difficulty': block.difficulty,
            'gas_limit': block.gasLimit,
            'gas_used': block.gasUsed,
            'date': datetime.datetime.fromtimestamp(block.timestamp).strftime('%Y-%m-%d %H:%M:%S')
        }

    # Get the latest block number
    latest_block = w3.eth.block_number

    # Fetch details of the latest 10 blocks
    blocks = [get_block(latest_block - i) for i in range(10)]

    # Convert to pandas DataFrame
    df = pl.DataFrame(blocks)

    # Save DataFrame to Parquet file
    df.write_parquet('ethereum_blocks.parquet')



# async def consume_messages():
#     logger.info("Starting to consume blocks from RabbitMQ.")
#     await consume_blocks()