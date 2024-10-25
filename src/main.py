import asyncio
from loguru import logger
import sys

from indexer import EVMIndexer
from rpc_types import ChainType

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
        # Setup indexer
        logger.info(f"Processing {CHAIN_NAME} chain")
        chain_type = ChainType(CHAIN_NAME)
        evm_indexer = EVMIndexer(rpc_url, chain_type)

        # Get current block number
        block_number = await evm_indexer.get_block_number()
        logger.info(f"Current block number: {block_number}")

        # block_number = 1

        # Get raw block and logs
        raw_block = await evm_indexer.get_block(block_number)
        # raw_logs = await evm_indexer.get_logs(block_number)
        
        # Parse block, transactions, and logs
        parsed_block = await evm_indexer.parse_block(raw_block)
        # parsed_transactions = await evm_indexer.parse_transactions(raw_block['transactions'])
        # parsed_logs = await evm_indexer.parse_logs(raw_logs)


        # Print or process the results
        # print("Parsed Block:", json.dumps(parsed_block, indent=2))
        # print(f"Parsed {len(parsed_transactions)} transactions")
        
        # # Optionally save to files
        # with open(f"data/temp-new/{CHAIN_NAME}/{CHAIN_NAME}_block_{block_number}.json", 'w') as f:
        #     json.dump(parsed_block, f, indent=4)
            
        # with open(f"data/temp-new/{CHAIN_NAME}/{CHAIN_NAME}_transactions_{block_number}.json", 'w') as f:
        #     json.dump(parsed_transactions, f, indent=4)

        # with open(f"data/temp-new/{CHAIN_NAME}/{CHAIN_NAME}_logs_{block_number}.json", 'w') as f:
        #     json.dump(logs, f, indent=4)

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
