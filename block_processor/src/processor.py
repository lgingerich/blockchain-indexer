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

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Global variable to track the next block number to be processed
next_block_to_process = 0

async def initialize_next_block_to_process(chain):
    global next_block_to_process
    next_block_to_process = find_highest_num_in_storage(storage_path=f'/app/data/{chain}/blocks')
    logger.info(f"Initialized with block number: {next_block_to_process}")

async def determine_next_block_to_process():
    global next_block_to_process

    # Increment the block number after processing
    next_block_to_process += 1
    logger.info(f"Next block to process: {next_block_to_process}")
    return next_block_to_process

async def get_blocks(block_number, w3):
    """
    Extract and return block information from the block number.
    """
    logger.info(f"Fetching block data for block number: {block_number}")

    try:
        block = w3.eth.get_block(block_number, full_transactions=True)
        print(block)
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
        # 'total_difficulty': Decimal(block.totalDifficulty),
        'block_time': datetime.datetime.utcfromtimestamp(block.timestamp).strftime('%Y-%m-%d %H:%M:%S'),
        'block_date': datetime.datetime.utcfromtimestamp(block.timestamp).strftime('%Y-%m-%d')
    }

    # Save full transaction data. Will throw error if the transaction field does not exist
    block_tx_data = block.transactions
    print(block_tx_data)
    logger.info(f"Block data fetched for block number: {block_number}")
    
    return block_data, block_tx_data


async def get_transactions(block_tx_data, w3):
    
    transaction_data = []
    log_data = []

    transactions = block_tx_data  

    for tx in transactions:
        try:
            # Process each transaction
            txs = {
                'access_list': tx['accessList'] if 'accessList' in tx else None,
                'block_hash': tx['blockHash'].hex(),
                'block_number': tx['blockNumber'],
                'chain_id': tx['chainId'],
                'from_address': tx['from'],
                'gas': tx['gas'],
                'gas_price': tx['gasPrice'],
                'input': tx['input'],
                'max_fee_per_gas': tx['maxFeePerGas'],
                'max_priority_fee_per_gas': tx['maxPriorityFeePerGas'],
                'nonce': tx['nonce'],
                'to_address': tx['to'],
                'transaction_hash': tx['hash'].hex(),           
                'transaction_index': tx['transactionIndex'],
                'type': tx['type'],
                'value': tx['value'],
                'v': tx['v'],
                'r': tx['r'].hex(),
                's': tx['s'].hex(),
                'y_parity': tx['yParity'],
            }

            # Fetch transaction receipt
            try:
                receipt = await w3.eth.get_transaction_receipt(tx['hash'])
            except TransactionNotFound as e:
                logger.error(f"Transaction receipt not found: {e}")
                continue
            except Web3Exception as e:
                logger.error(f"Web3 related error fetching receipt: {e}")
                continue

            receipt_data = {
                'contract_address': receipt.contractAddress,
                'cumulative_gas_used': receipt.cumulativeGasUsed,
                'effective_gas_price': receipt.effectiveGasPrice,
                'gas_used': receipt.gasUsed,
                'logs_bloom': receipt.logsBloom.hex(),
                'status': receipt.status
            }

            # Combine transaction data with receipt data
            txs.update(receipt_data)
            transaction_data.append(txs)

            # Process logs for each transaction
            logs_for_tx = get_logs(receipt)
            log_data.extend(logs_for_tx)

        except Exception as e:
            logger.error(f"Error processing transaction {tx['hash'].hex()}: {e}")
            continue

    return transaction_data, log_data


def get_logs(receipt):
    processed_logs = []

    for log in receipt.logs:
        try:
            topics = [topic.hex() for topic in log['topics']]
            processed_log = {
                'contract_address': log['address'],
                'block_hash': log['blockHash'].hex(),
                'block_number': log['blockNumber'],
                'data': log['data'],
                'log_index': log['logIndex'],
                'removed': log['removed'],
                'topic0': topics[0] if len(topics) > 0 else None,
                'topic1': topics[1] if len(topics) > 1 else None,
                'topic2': topics[2] if len(topics) > 2 else None,
                'topic3': topics[3] if len(topics) > 3 else None,
                'transaction_index': log['transactionIndex'],
                'transaction_hash': log['transactionHash'].hex(),
            }

            processed_logs.append(processed_log)

        except Exception as e:
            logger.error(f"Error processing log entry: {e}")
            break

    return processed_logs


async def process_data(RPC_URL_HTTPS, chain):
    global next_block_to_process
    await initialize_next_block_to_process(chain)

    # Set up HTTP RPC connection
    w3 = Web3.Web3(Web3.HTTPProvider(RPC_URL_HTTPS))

    # while True:
    while next_block_to_process <= 1650189:
        try:
            block_num_to_process = await determine_next_block_to_process()

            # Get blocks
            block_data, block_tx_data = await get_blocks(block_num_to_process, w3)
           
            if block_data:
                # Save blocks
                await save_data(block_data, chain, 'blocks')
            else:
                logger.warning(f"Block data is null for block number: {next_block_to_process}")


            if block_tx_data:
                # Get transactions and logs
                transaction_data, log_data = await get_transactions(block_tx_data, w3)

                # Save transactions
                await save_data(transaction_data, chain, 'transactions')

                if log_data:
                    # Save logs
                    await save_data(log_data, chain, 'logs')
                else:
                    logger.warning(f"Log data is null for transaction has: ###########")
            else:
                logger.warning(f"Transaction data is null for block number: {next_block_to_process}")

        except Exception as e:
            logger.error(f"Error in process_data: {e}")
            logger.error("Traceback:", exc_info=True)
            continue
