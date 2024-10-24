import asyncio
import json
import sys

from loguru import logger
from indexer import EVMIndexer
from rpc_types import ChainType
from web3 import Web3

# CHAIN_NAME = "arbitrum"
# CHAIN_NAME = "cronos-zkevm"
CHAIN_NAME = "ethereum"
# CHAIN_NAME = "zksync"

# rpc_url = "https://arbitrum.llamarpc.com"
# rpc_url = "https://mainnet.zkevm.cronos.org"
rpc_url = "https://eth.llamarpc.com"
# rpc_url = "https://mainnet.era.zksync.io"

async def main():
    try:

        chain_type = ChainType(CHAIN_NAME)

        evm_indexer = EVMIndexer(rpc_url, chain_type)

        block_number = await evm_indexer.get_block_number()
        # print(f"Current block number: {block_number}")

        # block_number = 1
        raw_block = await evm_indexer.get_block(block_number)
        block = await evm_indexer.process_block(raw_block)
        transactions = await evm_indexer.process_transactions(raw_block['transactions'])

        # print(f"Current block: {block}")

        # Convert block to JSON-serializable format
        block_json = Web3.to_json(block)

        # If block_json is a string, parse it back to a Python object
        if isinstance(block_json, str):
            block_json = json.loads(block_json)

        # Write directly to file
        # with open(f"data/schema-ref{CHAIN_NAME}/{CHAIN_NAME}_block_{block_number}.json", 'w') as file:
        with open(f"data/temp-new/{CHAIN_NAME}/{CHAIN_NAME}_block_{block_number}.json", 'w') as file:
            json.dump(block_json, file, indent=4, sort_keys=True)

        # Convert transactions to JSON-serializable format
        transactions_json = [Web3.to_json(tx) for tx in transactions]

        # If any transaction is a string, parse it back to a Python object
        transactions_json = [json.loads(tx) if isinstance(tx, str) else tx for tx in transactions_json]

        # Write transactions data to file
        with open(f"data/temp-new/{CHAIN_NAME}/{CHAIN_NAME}_transactions_{block_number}.json", 'w') as file:
            json.dump(transactions_json, file, indent=4, sort_keys=True)

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
