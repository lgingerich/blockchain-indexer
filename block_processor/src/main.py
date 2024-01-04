import asyncio
import os
from utils import setup_logging, get_config_value
# from database import init_db
from processor import process_data
import time 

# Access setup configurations

# print('---------------------------------------------------------')
CHAIN = os.getenv('CHAIN')
LOG_TO_FILE = os.getenv('LOG_TO_FILE')
RPC_URL_HTTPS = os.getenv('RPC_URL_HTTPS')

# print(CHAIN)
# print(LOG_TO_FILE)
# print(RPC_URL_HTTPS)

# print('=========================================================')
# CHAIN = get_config_value('chain.name')
# LOG_TO_FILE = get_config_value('log.to_file')
# RPC_URL_HTTPS = get_config_value('chain.rpc.https')

# print(CHAIN)
# print(LOG_TO_FILE)
# print(RPC_URL_HTTPS)

start_time = time.time()  # Start time

async def main():

    # Set up global logging configuration
    setup_logging(LOG_TO_FILE)

    # print('=================================================================')
    # print(get_config_value('log.destination'))
    # print('=================================================================')
    # Initialize data storage if necessary (create tables, etc.)
    # init_db()

    # Start the main processing logic
    await process_data(RPC_URL_HTTPS, CHAIN)

if __name__ == "__main__":
    asyncio.run(main())

end_time = time.time()  # End time
elapsed_time = end_time - start_time
print(f"Elapsed time: {elapsed_time} seconds")


"""
Notes:

- change .env to yml config file
- need to fix/organize data passing get_transactions and get_logs
- improve how i access transaction data?
- For L2s, need to update data after transaction settle on L1
    - every minute, check if certain columns are returned and update data if yes
- separate transaction receipt appending
- make data folder if it doesn't exist
- in processor.py, are there times I should throw an error if a certain field isn't returned?
- standardize warning vs error vs info logging
- some transactions don't have receipts
"""