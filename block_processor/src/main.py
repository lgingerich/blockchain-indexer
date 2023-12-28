import asyncio
from utils import setup_logging
from database import init_db
from processor import process_data
from web3 import Web3
import os

# Access environment variables
RPC_URL_HTTPS = os.getenv('RPC_URL_HTTPS')

async def main():

    # Set up HTTP RPC connection
    w3 = Web3(Web3.HTTPProvider(RPC_URL_HTTPS))

    # Set up global logging configuration
    setup_logging()

    # Initialize data storage if necessary (create tables, etc.)
    # init_db()

    # Start the main processing logic
    await process_data(w3)

if __name__ == "__main__":
    asyncio.run(main())
