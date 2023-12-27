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
from utils import find_highest_num_in_storage
from db.repository import BlockRepository
import logging
import datetime
from decimal import Decimal
from web3 import Web3
import polars as pl

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


# Global variable to track the next block number to be processed
next_block_to_process = 0

async def initialize_next_block_to_process():
    global next_block_to_process
    next_block_to_process = find_highest_num_in_storage(storage_path='/app/data/')
    logger.info(f"Initialized with block number: {next_block_to_process}")

async def determine_next_block_to_process():
    global next_block_to_process
    # Increment the block number after processing
    next_block_to_process += 1
    logger.info(f"Next block to process: {next_block_to_process}")
    return next_block_to_process

async def get_block_data(block_number):
    """
    Extract and return block information from the block data.
    """
    logger.info(f"Fetching block data for block number: {block_number}")

    # Connect to Ethereum node
    w3 = Web3(Web3.HTTPProvider('https://ethereum.publicnode.com'))
    
    try:
        block = w3.eth.get_block(block_number, full_transactions=True)
    except Exception as e:
        logger.error(f"Error fetching block data: {e}")
        raise

    block_data = {
        'base_fee_per_gas': block.baseFeePerGas if 'baseFeePerGas' in block else None,
        'difficulty': block.difficulty,
        'gas_limit': block.gasLimit,
        'gas_used': block.gasUsed,
        'hash': block.hash.hex(),
        'miner': block.miner,
        'nonce': block.nonce.hex(),
        'number': block.number,
        'parent_hash': block.parentHash.hex(),
        'size': block.size,
        'timestamp': block.timestamp,
        'total_difficulty': Decimal(block.totalDifficulty),
        'block_time': datetime.datetime.utcfromtimestamp(block.timestamp).strftime('%Y-%m-%d %H:%M:%S'),
        'block_date': datetime.datetime.utcfromtimestamp(block.timestamp).strftime('%Y-%m-%d')
    }

    logger.info(f"Block data fetched for block number: {block_number}")
    return block_data

# this currently saves only a single block and overwrites data 
async def save_data(data, chain, table):
    logger.info(f"Saving data to {chain}_{table}.")
    df = pl.DataFrame()
    df = pl.DataFrame(data)
    try:
        df.write_parquet(f'/app/data/{chain}_{table}.parquet')
    except Exception as e:
        logger.error(f"Error saving data: {e}")
        raise
    logger.info(f"Data saved successfully to {chain}_{table}.")


async def process_data():
    global next_block_to_process
    await initialize_next_block_to_process()

    while True:
        try:
            block_num_to_process = await determine_next_block_to_process()
            block_data = await get_block_data(block_num_to_process)
            await save_data(block_data, 'ethereum', 'blocks')
        except Exception as e:
            logger.error(f"Error in process_data: {e}")
            break


    # # Get the latest block number
    # latest_block = w3.eth.block_number
    # # latest_block = 1

    # # Fetch details of the latest 10 blocks
    # blocks = [get_block(latest_block - i) for i in range(10)]

    # # Convert to polars DataFrame
    # df = pl.DataFrame(blocks)

    # # Save DataFrame to Parquet file
    # df.write_parquet('/app/data/ethereum_blocks_2.parquet')
    # # df.write_csv('/app/data/ethereum_blocks.csv', separator=",")


# async def consume_messages():
#     logger.info("Starting to consume blocks from RabbitMQ.")
#     await consume_blocks()