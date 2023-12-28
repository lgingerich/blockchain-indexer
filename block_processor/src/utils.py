import logging
import os
import polars as pl

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def setup_logging(LOG_TO_FILE, default_level=logging.INFO):
    """
    Set up the logging configuration.
    """
    logging.basicConfig(
        level=default_level,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        datefmt='%Y-%m-%d %H:%M:%S'
    )

    if LOG_TO_FILE:
        file_handler = logging.FileHandler('/app/data/application.log')
        file_handler.setFormatter(logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s'))
        logging.getLogger().addHandler(file_handler)

    # Setup logging to save to cloud storage
        

def find_highest_num_in_storage(storage_path):
    highest_number = 0 # if no data exists, start from genesis

    for filename in os.listdir(storage_path):
        if filename.endswith('.parquet'):
            file_path = os.path.join(storage_path, filename)
            df = pl.read_parquet(file_path)
            
            if 'number' in df.columns:
                max_in_file = df['number'].max()
                highest_number = max(max_in_file, highest_number)

    return highest_number

async def save_data(data, chain, table):
    file_path = f'/app/data/{chain}_{table}.parquet'
    
    # Convert the data to a DataFrame
    try:
        new_df = pl.DataFrame(data)
    except Exception as e:
        logger.error(f"Error converting data to DataFrame: {e}")
        raise

    # Check if the file already exists
    if os.path.exists(file_path):
        # logger.info(f"Appending data to existing {chain}_{table} file.")
        try:
            # Read existing data
            existing_df = pl.read_parquet(file_path)
            # Append new data
            combined_df = pl.concat([existing_df, new_df])
        except Exception as e:
            logger.error(f"Error reading or appending existing data: {e}")
            raise
    else:
        # logger.info(f"Creating new {chain}_{table} file.")
        combined_df = new_df

    # Write combined data back to file
    try:
        combined_df.write_parquet(file_path)
    except Exception as e:
        logger.error(f"Error writing data to {file_path}: {e}")
        raise
    logger.info(f"Data successfully saved to {file_path}.")
