# utils.py

import logging
import os

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
