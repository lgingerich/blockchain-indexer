from consumer import consume_blocks
# from database import get_db
from utils import find_highest_num_in_storage, save_data
# from db.repository import BlockRepository
import logging
import datetime
from decimal import Decimal
import polars as pl
import web3 as Web3
from web3.exceptions import BlockNotFound, TransactionNotFound, Web3Exception
from concurrent.futures import ThreadPoolExecutor
import asyncio

executor = ThreadPoolExecutor(max_workers=4)  # Adjust the number of workers as needed

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Global variable to track the next block number to be processed
next_block_to_process = 0

async def initialize_next_block_to_process(CHAIN):
    global next_block_to_process
    next_block_to_process = find_highest_num_in_storage(storage_path=f'/app/data/{CHAIN}/blocks')
    next_block_to_process = 1650000
    logger.info(f"Initialized with block number: {next_block_to_process}")

async def determine_next_block_to_process():
    global next_block_to_process

    # Increment the block number after processing
    next_block_to_process += 1
    logger.info(f"Next block to process: {next_block_to_process}")
    return next_block_to_process

async def get_blocks(block_number, w3):
    """
    Extract information for each block number. Return block data and transaction data.
    """
    # logger.info(f"Fetching block data for block number: {block_number}")

    try:
        block = w3.eth.get_block(block_number, full_transactions=True)
    except BlockNotFound as e:
        logger.error(f"Block not found: {e}")
        return None, None
    except Web3Exception as e:
        logger.error(f"Web3 related error: {e}")
        return None, None
    except Exception as e:
        logger.error(f"Unexpected error fetching block data: {e}")
        return None, None

    block_data = {
        'base_fee_per_gas': block.get('baseFeePerGas', None),
        'difficulty': block.get('difficulty', None),
        'extra_data': block.get('extra_data', None).hex() if block.get('extra_data') is not None else None,
        'gas_limit': block.get('gasLimit', None),
        'gas_used': block.get('gasUsed'),
        'block_hash': block.get('hash', None).hex() if block.get('hash') is not None else None,
        'logs_bloom': block.get('logsBloom', None).hex() if block.get('logsBloom') is not None else None,
        'miner': block.get('miner', None),
        'mix_hash': block.get('mixHash', None).hex() if block.get('mixHash') is not None else None,
        'nonce': block.get('nonce').hex() if block.get('nonce') is not None else None,
        'number': block.get('number'),
        'parent_hash': block.get('parentHash').hex() if block.get('parentHash') is not None else None,
        'receipts_root': block.get('receiptsRoot').hex() if block.get('receiptsRoot') is not None else None,
        'sha3_uncles': block.get('sha3Uncles').hex() if block.get('sha3Uncles') is not None else None,
        'size': block.get('size'),
        'state_root': block.get('stateRoot').hex() if block.get('stateRoot') is not None else None,
        'timestamp': block.get('timestamp'),
        # 'total_difficulty': Decimal(block.totalDifficulty),
        'block_time': datetime.datetime.utcfromtimestamp(block.get('timestamp')).strftime('%Y-%m-%d %H:%M:%S') if block.get('timestamp') is not None else None,
        'block_date': datetime.datetime.utcfromtimestamp(block.get('timestamp')).strftime('%Y-%m-%d') if block.get('timestamp') is not None else None
    }

    # Save full transaction data
    block_tx_data = block.get('transactions', [])

    logger.info(f"Block data fetched for block number: {block_number}")
    
    return block_data, block_tx_data

async def get_transactions(block_tx_data):
    """
    Extract information for each transaction from data returned by get_blocks(). 
    Return cleaned transaction data.
    """
    transaction_data = []

    for tx in block_tx_data:
        try:
            txs = {
                'block_hash': tx.get('blockHash', None).hex() if tx.get('blockHash') is not None else None,
                'block_number': tx.get('blockNumber', None),
                'from_address': tx.get('from', None),
                'to_address': tx.get('to', None),
                'gas_limit': tx.get('gas', None),
                'gas_price': tx.get('gasPrice', None),
                'max_fee_per_gas': tx.get('maxFeePerGas', None),
                'max_priority_fee_per_gas': tx.get('maxPriorityFeePerGas', None),
                'transaction_hash': tx.get('hash', None).hex() if tx.get('hash') is not None else None,
                'input': tx.get('input', None).hex() if tx.get('input') is not None else None,
                'nonce': tx.get('nonce', None),
                'transaction_index': tx.get('transactionIndex', None),
                'value': tx.get('value', None),
                'type': tx.get('type', None),
                'access_list': tx.get('accessList', None),
                'chain_id': tx.get('chainId', None),
                'v': tx.get('v', None),
                'r': tx.get('r', None).hex() if tx.get('r') is not None else None,
                's': tx.get('s', None).hex() if tx.get('s') is not None else None,
                'y_parity': tx.get('yParity', None),
            }

            transaction_data.append(txs)

        except Exception as e:
            logger.error(f"Error processing transaction {tx.get('hash', None).hex() if tx.get('hash') is not None else 'unknown'}: {e}")
            continue

    return transaction_data

async def get_logs(block_data, w3):
    """
    Extract information for each log from data returned by get_blocks().
    Return cleaned log data.
    """
    log_data = []

    block_hash = block_data.get('block_hash', None)

    if block_hash:
        try:
            logs = w3.eth.get_logs({'blockHash': block_hash})
        except BlockNotFound as e:
            logger.error(f"Block not found: {e}")
            return None, None
        except Web3Exception as e:
            logger.error(f"Web3 related error: {e}")
            return None, None
        except Exception as e:
            logger.error(f"Unexpected error fetching block data: {e}")
            return None, None
        
        for log in logs:
            try:
                topics = [topic.hex() for topic in log.get('topics', [])]
                processed_log = {
                    'contract_address': log.get('address'),
                    'block_hash': log.get('blockHash', None).hex() if log.get('blockHash') is not None else None,
                    'block_number': log.get('blockNumber', None),
                    'data': log.get('data'),
                    'log_index': log.get('logIndex', None),
                    'removed': log.get('removed', None),
                    'topic0': topics[0] if len(topics) > 0 else None,
                    'topic1': topics[1] if len(topics) > 1 else None,
                    'topic2': topics[2] if len(topics) > 2 else None,
                    'topic3': topics[3] if len(topics) > 3 else None,
                    'transaction_index': log.get('transactionIndex', None),
                    'transaction_hash': log.get('transactionHash', None).hex() if log.get('transactionHash') is not None else None,
                }

                log_data.extend(processed_log)

            except Exception as e:
                logger.error(f"Error processing log entry: {e}")
                continue

    return log_data

async def get_transaction_receipts(transactions, w3):
    """
    Extract and return transactions receipts from the transaction hash.
    """

    tx_receipts = []

    for tx in transactions:
        transaction_hash = tx.get('transaction_hash')
        if not transaction_hash:
            logger.error("Transaction hash not found in the transaction data")
            continue

        try:
            receipt = w3.eth.get_transaction_receipt(transaction_hash)
        except TransactionNotFound as e:
            logger.error(f"Transaction receipt not found: {e}")
            continue
        except Web3Exception as e:
            logger.error(f"Web3 related error fetching receipt: {e}")
            continue

        txs = {
                'block_hash': receipt.get('blockHash', None).hex() if receipt.get('blockHash') is not None else None,
                'block_number': receipt.get('blockNumber', None),
                'contract_address': receipt.get('contractAddress', None),
                'cumulative_gas_used': receipt.get('cumulativeGasUsed', None),
                'effective_gas_price': receipt.get('effectiveGasPrice', None),
                'from_address': receipt.get('from', None),
                'gas_used': receipt.get('gasUsed', None),
                'logs_bloom': receipt.get('logsBloom', None).hex() if receipt.get('logsBloom') is not None else None,
                'status': receipt.get('status', None),
                'to_address': receipt.get('to', None),
                'transaction_hash': receipt.get('transactionHash', None).hex() if receipt.get('transactionHash') is not None else None,
                'transaction_index': receipt.get('transactionIndex', None),
                'type': receipt.get('type', None),
        }

        tx_receipts.append(txs)

    return tx_receipts


async def process_data(RPC_URL_HTTPS, CHAIN, CHUNK_SIZE):
    global next_block_to_process

    # Initialize separate dataframes for blocks, transactions, and logs
    block_df = pl.DataFrame()
    transaction_df = pl.DataFrame()
    log_df = pl.DataFrame()
    
    first_block_in_batch = None

    await initialize_next_block_to_process(CHAIN)

    # Set up HTTP RPC connection
    w3 = Web3.Web3(Web3.HTTPProvider(RPC_URL_HTTPS))

    # while True:
    while next_block_to_process <= 1650500:
        try:
            block_num_to_process = await determine_next_block_to_process()
            
            # logger.info(f"block_num_to_process: {block_num_to_process}")
            # Initialize first_block_in_batch for the first iteration
            if first_block_in_batch is None:
                first_block_in_batch = block_num_to_process
                # logger.info(f"first_block_in_batch: {first_block_in_batch}")

            # Get block and transaction data
            block_data, block_tx_data = await get_blocks(block_num_to_process, w3)

            if block_data:
                # Append block data
                block_df = block_df.vstack(pl.DataFrame(block_data))

                # Get logs
                log_data = await get_logs(block_data, w3)
                if log_data:
                    # Append log data
                    log_df = log_df.vstack(pl.DataFrame(log_data))

            if block_tx_data:
                # Get transactions
                transaction_data = await get_transactions(block_tx_data)
                if transaction_data:
                    # Append transaction data
                    transaction_df = transaction_df.vstack(pl.DataFrame(transaction_data))

            # Check if the interval is reached. If yes, save data
            if (block_num_to_process - first_block_in_batch + 1) >= CHUNK_SIZE:

                # Save blocks
                loop = asyncio.get_running_loop()
                loop.run_in_executor(executor, save_data, block_df, CHAIN, f'blocks_{first_block_in_batch}_{block_num_to_process}')

                # Save transactions
                loop = asyncio.get_running_loop()
                loop.run_in_executor(executor, save_data, transaction_df, CHAIN, f'transactions_{first_block_in_batch}_{block_num_to_process}')

#               # Save logs
                loop = asyncio.get_running_loop()
                loop.run_in_executor(executor, save_data, log_df, CHAIN, f'logs_{first_block_in_batch}_{block_num_to_process}')

                # Reset the dataframes for the next set of blocks
                block_df = pl.DataFrame()
                transaction_df = pl.DataFrame()
                log_df = pl.DataFrame()
                first_block_in_batch = None

        except Exception as e:
            logger.error(f"Error in process_data: {e}")
            logger.error("Traceback:", exc_info=True)
            continue
