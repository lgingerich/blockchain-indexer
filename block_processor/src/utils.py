import logging
import os
import polars as pl

def setup_logging(default_level=logging.INFO):
    """
    Set up the logging configuration.
    """
    logging.basicConfig(
        level=default_level,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        datefmt='%Y-%m-%d %H:%M:%S'
    )

    # Example of how to configure logging to a file
    if os.getenv('LOG_TO_FILE'):
        file_handler = logging.FileHandler('application.log')
        file_handler.setFormatter(logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s'))
        logging.getLogger().addHandler(file_handler)

    # If needed, set up other loggers or handlers (e.g., for external libraries)

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
