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
from .consumer import consume_blocks
from .database import get_db
from db.repository import BlockRepository
import logging

logger = logging.getLogger(__name__)

# async def determine_next_block_to_process(latest_block_in_queue):
async def determine_next_block_to_process():
    with get_db() as db:
        latest_block = BlockRepository.get_latest_block(db)
        if latest_block:
            latest_block_number = latest_block.number
            # Now, use latest_block_number for your logic
        else:
            # Handle the case when there are no blocks in the database
            print('1')


# async def process_data():
#     # Logic for processing data
#     # ...

# async def consume_messages():
#     logger.info("Starting to consume blocks from RabbitMQ.")
#     await consume_blocks()