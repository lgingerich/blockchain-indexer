import asyncio
import os
from utils import setup_logging
from database import init_db
from processor import process_data

# Access environment variables
CHAIN = os.getenv('CHAIN')
LOG_TO_FILE = os.getenv('LOG_TO_FILE')
RPC_URL_HTTPS = os.getenv('RPC_URL_HTTPS')

async def main():

    # Set up global logging configuration
    setup_logging(LOG_TO_FILE)

    # Initialize data storage if necessary (create tables, etc.)
    # init_db()

    # Start the main processing logic
    await process_data(RPC_URL_HTTPS, CHAIN)

if __name__ == "__main__":
    asyncio.run(main())
