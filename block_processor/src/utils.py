import logging
import os
import polars as pl
import yaml

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def setup_logging(LOG_TO_FILE, LOG_DESTINATION, default_level=logging.INFO):
    """
    Set up the logging configuration.
    """
    logging.basicConfig(
        level=default_level,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        datefmt='%Y-%m-%d %H:%M:%S'
    )

    if LOG_TO_FILE:
        file_handler = logging.FileHandler('/app' + LOG_DESTINATION)
        file_handler.setFormatter(logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s'))
        logging.getLogger().addHandler(file_handler)

    # Setup logging to save to cloud storage
        
def get_config_value(config_path):
    """
    Retrieves a specific configuration value based on a dot-notated path.
    Automatically handles special keys for sections like 'data' and 'logging' based on their 'type'.

    :param config_path: A string in the format 'section.key' or 'section.subsection.key'.
    :return: The value of the configuration setting or None if not found.
    """

    # Define keys that require special handling based on 'type'
    special_keys = {'destination'}

    # Get the directory of the current script (utils.py)
    current_dir = os.path.dirname(__file__)

    # Go up two directories from the current script
    parent_dir = os.path.dirname(os.path.dirname(current_dir))

    # Construct the path to config.yml
    file_path = os.path.join(parent_dir, 'config.yml')

    try:
        with open(file_path, 'r') as file:
            config = yaml.safe_load(file)

        # Split the path to get section and key
        path_parts = config_path.split('.')
        if len(path_parts) < 2:
            raise ValueError("Config path must be in the format 'section.key' or 'section.subsection.key'")

        section = path_parts[0]
        key = path_parts[-1]
        section_config = config.get(section, {})

        # Navigate through subsections if any
        for part in path_parts[1:-1]:
            section_config = section_config.get(part, {})

        # Check if the key is one of the special keys and handle accordingly
        if key in special_keys and section in ['data', 'logging']:
            type_key = section_config.get('type')
            if type_key not in ['local', 'cloud']:
                raise ValueError(f"Invalid type for section {section}. Must be 'local' or 'cloud'.")

            return section_config.get(type_key, {}).get(key)

        # For other sections or keys, handle normally
        return section_config.get(key)
    except Exception as e:
        print(f"Error processing config file: {e}")
        return None


def find_highest_num_in_storage(storage_path):
    highest_number = 0  # if no data exists, start from genesis
    
    # Check if the storage_path exists
    if not os.path.exists(storage_path):
        return highest_number

    for filename in os.listdir(storage_path):
        if filename.endswith('.parquet'):
            file_path = os.path.join(storage_path, filename)
            try:
                df = pl.read_parquet(file_path)
                if 'number' in df.columns:
                    max_in_file = df['number'].max()
                    highest_number = max(max_in_file, highest_number)
            except Exception as e:
                logger.error(f"Error reading file {filename}: {e}")

    return highest_number

def save_data(data, chain, table):
    file_path = f'/app/data/{chain}/{table}/{table}.parquet'
    
    # Ensure the directory exists
    os.makedirs(os.path.dirname(file_path), exist_ok=True)

    # # Convert the data to a DataFrame
    # try:
    #     new_df = pl.DataFrame(data)
    # except Exception as e:
    #     logger.error(f"Error converting data to DataFrame: {e}")
    #     raise

    # # Check if the file already exists
    # if os.path.exists(file_path):
    #     logger.info(f"Appending data to existing {chain}/{table}/{table} file.")
    #     try:
    #         # Attempt to read existing data
    #         try:
    #             existing_df = pl.read_parquet(file_path)
    #         except Exception as e:
    #             logger.error(f"Error reading existing Parquet file: {e}")
    #             # Handle the error (e.g., skip appending, log the issue, etc.)
    #             # For example, just use new data
    #             combined_df = new_df
    #             # Optionally, you can skip the rest of this iteration with 'continue'
    #         else:
    #             # Append new data if existing data is successfully read
    #             combined_df = pl.concat([existing_df, new_df])
    #     except Exception as e:
    #         logger.error(f"Error appending data: {e}")
    #         raise
    # else:
    #     logger.info(f"Creating new {chain}/{table}/{table} file.")
    #     combined_df = new_df

    # Write combined data back to file
    try:
        data.write_parquet(file_path)
    except Exception as e:
        logger.error(f"Error writing data to {file_path}: {e}")
        raise
    logger.info(f"Data successfully saved to {file_path}.")