from hexbytes import HexBytes
from loguru import logger
from typing import Tuple, Union, List
from google.cloud import bigquery
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

def get_bigquery_schema(model_class) -> List[bigquery.SchemaField]:
    # Dictionary to map Python/Pydantic types to BigQuery types
    TYPE_MAPPING = {
        int: 'INTEGER',
        str: 'STRING',
        float: 'FLOAT',
        bool: 'BOOLEAN',
        datetime: 'TIMESTAMP',
        date: 'DATE',
    }

    schema = []
    
    # Get all fields including from parent classes
    all_annotations = {}
    for cls in model_class.__mro__:
        if hasattr(cls, '__annotations__'):
            all_annotations.update(cls.__annotations__)
    
    for field_name, field in all_annotations.items():
        # Skip internal fields and config
        if field_name.startswith('_') or field_name == 'model_config':
            continue
            
        # Check if field is a generic type (e.g. List, Optional, Union, Dict, Tuple, etc.)
        if hasattr(field, "__origin__"):
            if field.__origin__ == list:
                # Get the type of the list elements
                inner_type = field.__args__[0]
                bq_type = TYPE_MAPPING.get(inner_type, 'STRING') # Default to STRING if type is not found
                schema_field = bigquery.SchemaField(
                    name=field_name,
                    field_type=bq_type,
                    mode='REPEATED'
                )
            else:
                field_type = field.__args__[0]
                bq_type = TYPE_MAPPING.get(field_type, 'STRING') # Default to STRING if type is not found
                schema_field = bigquery.SchemaField(
                    name=field_name,
                    field_type=bq_type,
                    mode='NULLABLE'
                )
        # If field is not a generic type, just use the type directly
        # All optional fields are defined with the type Optional[type]. Any optional
        # fields are handled above with the generic type check.
        else:
            field_type = field
            bq_type = TYPE_MAPPING.get(field_type, 'STRING') # Default to STRING if type is not found
            schema_field = bigquery.SchemaField(
                name=field_name,
                field_type=bq_type,
                mode='REQUIRED'
            )
        
        schema.append(schema_field)
    
    return schema