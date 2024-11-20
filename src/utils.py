import asyncio
from datetime import datetime, timezone, date
from dynaconf import Dynaconf, Validator
from functools import wraps
from hexbytes import HexBytes
from loguru import logger
from pathlib import Path
import random
from typing import Union


def hex_to_str(hex_value: HexBytes) -> str:
    # Ensure input is HexBytes type
    if not isinstance(hex_value, HexBytes):
        raise TypeError(f"Expected HexBytes, got {type(hex_value)}")
    
    # Convert to hex string, maintaining '0x' prefix
    return '0x' + hex_value.hex()

def unix_to_utc(timestamp: int, date_only: bool = False) -> Union[date, datetime]:
    """Convert Unix timestamp to UTC datetime/date object
    
    Args:
        timestamp (int): Unix timestamp in seconds
        date_only (bool): If True, returns date object. 
                         If False, returns datetime object
        
    Returns:
        Union[date, datetime]: UTC datetime or date object
    """
    dt = datetime.fromtimestamp(timestamp, timezone.utc)
    return dt.date() if date_only else dt

def load_config(file_name: str) -> Dynaconf:
    """Load and validate indexer configuration from chain config file
    
    Ensures that only one chain configuration is active.

    Params:
        file_name (str): Name of the chain config file to load

    Returns:
        Dynaconf: Validated configuration object
    """
    # Initialize Dynaconf
    project_root = Path(__file__).resolve().parent.parent
    config_path = project_root / "chains" / file_name

    # Validate that only one 'chain' section is active
    active_chain_count = 0
    with config_path.open('r') as f:
        for line in f:
            stripped_line = line.strip()
            # Check if the line starts with 'chain:' and is not commented out
            if stripped_line.startswith('chain:') and not line.lstrip().startswith('#'):
                active_chain_count += 1
                if active_chain_count > 1:
                    raise ValueError("Configuration Error: Multiple active 'chain' sections found in config.yml. Please ensure only one 'chain' configuration is active.")

    settings = Dynaconf(
        settings_files=[config_path],
        validators=[
            # Validate structure and types
            Validator('chain.name', must_exist=True, 
                     is_type_of=str,
                     condition=lambda x: x.islower() and x == x.strip(),
                     messages={"condition": "Chain name must be lowercase with no leading/trailing spaces"}
            ),
            Validator('chain.rpc_urls', must_exist=True, is_type_of=list),
        ]
    )
    # Validate all settings at once
    settings.validators.validate()
    
    return settings

# Decorator for implementing retry logic with exponential backoff for async functions
def async_retry(
    retries: int = 3,
    base_delay: int = 1,
    exponential_backoff: bool = True,
    jitter: bool = True,
):
    """
    Decorator for implementing retry logic with exponential backoff for async functions.

    :param retries: int, number of retry attempts
    :param base_delay: int, base delay between retries in seconds
    :param exponential_backoff: bool, whether to use exponential backoff
    :param jitter: bool, whether to add random jitter to the delay
    :return: function, decorated function
    """
    def decorator(func):
        @wraps(func)
        async def wrapper(*args, **kwargs):
            for attempt in range(1, retries + 1):
                try:
                    return await func(*args, **kwargs)
                except Exception as e:
                    if attempt == retries:
                        logger.error(
                            f"All retry attempts failed for {func.__name__}: {str(e)}"
                        )
                        raise
                    
                    delay = (
                        base_delay * (2 ** (attempt - 1))
                        if exponential_backoff
                        else base_delay
                    )
                    if jitter:
                        delay *= random.uniform(1.0, 1.5)

                    logger.warning(
                        f"Attempt {attempt} failed for {func.__name__}. Retrying in {delay:.2f} seconds. Error: {str(e)}"
                    )
                    await asyncio.sleep(delay)

        return wrapper
    return decorator