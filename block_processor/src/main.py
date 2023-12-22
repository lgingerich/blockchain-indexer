
"""
1. initialize tables if they don't exist
2. determine where to start indexing from
    - if no data exists, start from genesis
    - if some data exists, start from latest block
        - overwrite the latest block to ensure I got all transactions and logs
3. 






"""


import asyncio
from utils import setup_logging
from database import init_db
# from processor import start_processing


async def main():
    print('================================================================')
    
    # Set up global logging configuration
    setup_logging()

    # Initialize the database (create tables, etc.)
    init_db()

    # # Start the main processing logic
    # await start_processing()

if __name__ == "__main__":
    asyncio.run(main())
