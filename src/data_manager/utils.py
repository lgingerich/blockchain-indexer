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

    # Special column names that should use NUMERIC type
    NUMERIC_COLUMNS = {'difficulty', 'total_difficulty'}

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
            
        # Override type for specific column names
        if field_name in NUMERIC_COLUMNS:
            bq_type = 'NUMERIC'
        else:
            # Check if field is a generic type (e.g. List, Optional, Union, Dict, Tuple, etc.)
            if hasattr(field, "__origin__"):
                if field.__origin__ == list:
                    inner_type = field.__args__[0]
                    bq_type = TYPE_MAPPING.get(inner_type, 'STRING')
                else:
                    field_type = field.__args__[0]
                    bq_type = TYPE_MAPPING.get(field_type, 'STRING')
            else:
                field_type = field
                bq_type = TYPE_MAPPING.get(field_type, 'STRING')

        # Create schema field with appropriate mode
        if hasattr(field, "__origin__"):
            if field.__origin__ == list:
                mode = 'REPEATED'
            else:
                mode = 'NULLABLE'
        else:
            mode = 'REQUIRED'

        schema_field = bigquery.SchemaField(
            name=field_name,
            field_type=bq_type,
            mode=mode
        )
        
        schema.append(schema_field)
    
    return schema