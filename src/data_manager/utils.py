from typing import List
from google.cloud import bigquery
from datetime import datetime, date

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