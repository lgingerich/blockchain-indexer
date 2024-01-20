import asyncio
import os
from utils import setup_logging, get_config_value
from processor import process_data
import time 
import cProfile
import traceback

# Access setup configurations
CHAIN = get_config_value('chain.name')
LOG_TO_FILE = get_config_value('logging.to_file')
LOG_DESTINATION = get_config_value('logging.destination')
RPC_URL_HTTPS = get_config_value('chain.rpc.https')
CHUNK_SIZE = get_config_value('data.chunk_size')


start_time = time.time()  # Start time

async def main():

    # Set up global logging configuration
    setup_logging(LOG_TO_FILE, LOG_DESTINATION)

    # Start the main processing logic
    profiler = cProfile.Profile()
    profiler.enable()  # Start profiling
    
    await process_data(RPC_URL_HTTPS, CHAIN, CHUNK_SIZE)

    profiler.disable()  # Stop profiling
    profiler.dump_stats('/app/data/profiling_stats')  # Save the stats to a file


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except Exception as e:
        print("An error occurred:")
        traceback.print_exc()

end_time = time.time()  # End time
elapsed_time = end_time - start_time
print(f"Elapsed time: {elapsed_time} seconds")

"""
Notes:

- For L2s, need to update data after transaction settle on L1
    - every minute, check if certain columns are returned and update data if yes
- separate transaction receipt appending
- make data folder if it doesn't exist. need to make whatever folder the user specifies
- in processor.py, are there times I should throw an error if a certain field isn't returned?
- standardize warning vs error vs info logging
- some transactions don't have receipts
- fix data save location. currently combines config var and combining with chain name
- should I batch get_logs over all blocks in a "100 block" batch?
- figure out how to handle end block config vs. real-time/historical. also starting from existing data
- maybe get rid of all np.nan. it does not behave the same as 'None'
"""