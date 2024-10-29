from hexbytes import HexBytes
from loguru import logger
from typing import Tuple, Union
import asyncio
import random
from functools import wraps
from datetime import datetime, timezone, date

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