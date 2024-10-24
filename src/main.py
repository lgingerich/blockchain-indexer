import asyncio
import sys

from loguru import logger
from indexer import EVMIndexer
from rpc_types import ChainType

CHAIN_NAME = "zksync"
# CHAIN_NAME = "ethereum"

rpc_url = "https://zksync.meowrpc.com"
# rpc_url = "https://eth.llamarpc.com"

async def main():
    try:

        chain_type = ChainType(CHAIN_NAME)

        evm_indexer = EVMIndexer(rpc_url, chain_type)

        block_number = await evm_indexer.get_block_number()
        print(f"Current block number: {block_number}")

        block = await evm_indexer.get_block(block_number)
        print(f"Current block: {block}")

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