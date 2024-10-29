import asyncio
import json
import os
from loguru import logger
import sys
import pandas as pd
from dotenv import load_dotenv
from indexer import EVMIndexer
from rpc_types import ChainType
from eth_typing import BlockNumber
from typing import cast
from data_manager import BigQueryManager

# Chain names must be lower case and use underscores instead of hyphens
# TO DO: Once I move chain names to a config file, add automated checks on this

# CHAIN_NAME = "arbitrum"
# CHAIN_NAME = "cronos_zkevm"
# CHAIN_NAME = "ethereum"
# CHAIN_NAME = "zksync"
CHAIN_NAME = "zksync_sepolia"

# rpc_url = "https://arbitrum.llamarpc.com"
# rpc_url = "https://mainnet.zkevm.cronos.org"
# rpc_url = "https://eth.llamarpc.com"
# rpc_url = "https://mainnet.era.zksync.io"
rpc_url = "https://sepolia.era.zksync.dev"

load_dotenv()

CREDS_FILE_PATH = os.getenv("CREDS_FILE_PATH")
if CREDS_FILE_PATH is None:
    raise ValueError("CREDS_FILE_PATH environment variable is not set")

async def main():
    try:
        # Setup indexer
        logger.info(f"Processing {CHAIN_NAME} chain")
        chain_type = ChainType(CHAIN_NAME)
        evm_indexer = EVMIndexer(rpc_url, chain_type)

        # Initialize BigQuery manager
        bq_manager = BigQueryManager(
            credentials_path=CREDS_FILE_PATH,
            dataset_id=f"{CHAIN_NAME}"
        )

        # Get current block number
        block_number = await evm_indexer.get_block_number()
        logger.info(f"Current block number: {block_number}")

        block_number = 1
        # block_number -= 100_000

        # while True:

        # Get raw block and logs
        raw_block = await evm_indexer.get_block(cast(BlockNumber, block_number))
        # print(raw_block)
        # raw_logs = await evm_indexer.get_logs(cast(BlockNumber, block_number))

        # if raw_block is None or raw_logs is None:
        #     logger.error(f"Failed to fetch data for block {block_number}")
        #     # continue

        # Parse block, transactions, and logs
        parsed_block = await evm_indexer.parse_block(raw_block)
        # parsed_transactions = await evm_indexer.parse_transactions(raw_block['transactions'])
        # parsed_logs = await evm_indexer.parse_logs(raw_logs)

        block_dict = dict(parsed_block)  # TO DO: Move this conversion elsewhere
        df = pd.DataFrame([block_dict])
        print(df)
        print(df.dtypes)



        # Create table and load data
        bq_manager.create_and_load_table(
            df=df,
            table_id=f"blocks",
            if_exists='replace'
        )


        # block_number += 1
        # print(parsed_block)

        # Print or process the results
        # print("Parsed Block:", json.dumps(parsed_block, indent=2))
        # print(f"Parsed {len(parsed_transactions)} transactions")
        
        # # Optionally save to files
        # with open(f"data/temp-new/{CHAIN_NAME}/{CHAIN_NAME}_block_{block_number}.json", 'w') as f:
        #     json.dump(parsed_block, f, indent=4)
            
        # with open(f"data/temp-new/{CHAIN_NAME}/{CHAIN_NAME}_transactions_{block_number}.json", 'w') as f:
        #     json.dump(parsed_transactions, f, indent=4)

        # with open(f"data/temp-new/{CHAIN_NAME}/{CHAIN_NAME}_logs_{block_number}.json", 'w') as f:
        #     json.dump(parsed_logs, f, indent=4)
        # with open(f"data/schema-ref/{CHAIN_NAME}/{CHAIN_NAME}_logs_{block_number}.json", 'w') as f:
        #     json.dump(raw_logs, f, indent=4)

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
