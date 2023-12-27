
"""
1. start logging
2. initialize tables if they don't exist
3. determine where to start indexing from
    - if no data exists, start from genesis
    - if some data exists, start from latest block
        - overwrite the latest block to ensure I got all transactions and logs
3. 

"""


"""
TO-DO:

- Abstract date type conversions by export type.
"""



import asyncio
from utils import setup_logging
from database import init_db
from processor import process_data


async def main():

    # Set up global logging configuration
    setup_logging()

    # Initialize the database (create tables, etc.)
    init_db()

    # Start the main processing logic
    await process_data()

if __name__ == "__main__":
    asyncio.run(main())
