from google.cloud import bigquery
from google.oauth2 import service_account
import pandas as pd
from typing import Optional, Dict, List
from loguru import logger
import google.api_core.exceptions
from utils import get_bigquery_schema
from rpc_types import (
    ChainType,
    BLOCK_TYPE_MAPPING,
    TRANSACTION_TYPE_MAPPING,
    LOG_TYPE_MAPPING
)

class BigQueryManager:
    """
    A class to manage BigQuery operations for blockchain data
    """
    
    def __init__(self, credentials_path: str, chain_name: str):
        """
        Initialize BigQuery client with credentials and dataset
        
        Args:
            credentials_path (str): Path to service account JSON credentials file
            dataset_id (str): ID of the dataset to work with
        """
        self.credentials = service_account.Credentials.from_service_account_file(
            credentials_path,
            scopes=["https://www.googleapis.com/auth/cloud-platform"]
        )
        self.client = bigquery.Client(
            credentials=self.credentials,
            project=self.credentials.project_id
        )
        self.dataset_id = chain_name
        
        # Generate schemas dynamically from Pydantic models
        self.block_schema = get_bigquery_schema(BLOCK_TYPE_MAPPING[ChainType(chain_name)])
        self.transaction_schema = get_bigquery_schema(TRANSACTION_TYPE_MAPPING[ChainType(chain_name)])
        self.log_schema = get_bigquery_schema(LOG_TYPE_MAPPING[ChainType(chain_name)])

        # Create dataset if it doesn't exist on client initialization
        self.create_dataset(self.dataset_id, location="US")
        
        # Create tables if they don't exist on client initialization
        for table_id in ['blocks', 'transactions', 'logs']:
            schema = self._get_schema_for_table(table_id)
            self.create_table(table_id, schema)

    def _get_schema_for_table(self, table_id: str) -> List[bigquery.SchemaField]:
        """Helper method to get the appropriate schema for a table"""
        if table_id == 'blocks':
            return self.block_schema
        elif table_id == 'transactions':
            return self.transaction_schema
        elif table_id == 'logs':
            return self.log_schema
        else:
            raise ValueError(f"Unable to determine schema for table {table_id}")

    def create_dataset(self, dataset_id: str, location: str = "US") -> None:
        """
        Creates the dataset if it doesn't already exist
        """
        dataset_ref = self.client.dataset(dataset_id)
        
        try:
            self.client.get_dataset(dataset_ref)
            logger.info(f"Dataset {dataset_id} already exists")
        except Exception:
            # Dataset does not exist, create it
            dataset = bigquery.Dataset(dataset_ref)
            dataset.location = location
            dataset = self.client.create_dataset(dataset)
            logger.info(f"Created dataset {dataset_id} in location {location}")

    def create_table(self, table_id: str, schema: List[bigquery.SchemaField]) -> None:
        """
        Creates a table with the specified schema if it doesn't exist
        
        Args:
            table_id (str): ID for the table
            schema (List[bigquery.SchemaField]): Schema definition for the table
        """
        table_ref = self.client.dataset(self.dataset_id).table(table_id)
        
        try:
            self.client.get_table(table_ref)
            logger.info(f"Table {table_id} already exists")
            return
        except google.api_core.exceptions.NotFound:
            table = bigquery.Table(table_ref, schema=schema)
            
            # Add date partitioning
            partition_field = "block_date"
            table.time_partitioning = bigquery.TimePartitioning(
                type_=bigquery.TimePartitioningType.DAY,
                field=partition_field
            )
            
            table = self.client.create_table(table)
            logger.info(f"Created table {table.project}.{table.dataset_id}.{table.table_id} with partitioning on {partition_field}")

    def load_table(self, 
        df: pd.DataFrame, 
        table_id: str, 
        if_exists: str = 'append',
        chunk_size: int = 10000) -> None:
        """
        Load table data into a BigQuery table with specified handling for existing tables
        
        Args:
            df (pd.DataFrame): DataFrame containing the data to load
            table_id (str): ID of the target table
            if_exists (str): How to handle existing tables:
                - 'fail': Raise an error if table exists
                - 'replace': Drop existing table and create new one
                - 'append': Add data to existing table (default)
            chunk_size (int): Number of rows to load in each batch
        
        Raises:
            ValueError: If table exists and if_exists='fail' or if table_id is invalid
        """
        table_ref = self.client.dataset(self.dataset_id).table(table_id)
        
        try:
            existing_table = self.client.get_table(table_ref)
            if if_exists == 'fail':
                raise ValueError(f"Table {table_id} already exists")
            elif if_exists == 'replace':
                self.client.delete_table(table_ref)
                self.create_table(table_id, self._get_schema_for_table(table_id))
            # For 'append', we just continue to data loading
        except google.api_core.exceptions.NotFound:
            self.create_table(table_id, self._get_schema_for_table(table_id))
        except Exception as e:
            logger.error(f"Unexpected error while checking table {table_id}: {str(e)}")
            raise
        
        self._load_dataframe(df, table_ref, chunk_size)

    def _load_dataframe(self, df: pd.DataFrame, table_ref: str, chunk_size: int) -> None:
        """
        Load DataFrame into existing table using efficient batch loading
        
        Args:
            df (pd.DataFrame): DataFrame containing the data to load
            table_ref (str): Reference to the target table
            chunk_size (int): Number of rows to load in each batch
        """
        # Get the table ID from the TableReference object
        table_id = table_ref.table_id
        
        # Update to use the correct schema based on table name
        schema = None
        if table_id == 'blocks':
            schema = self.block_schema
        elif table_id == 'transactions':
            schema = self.transaction_schema
        elif table_id == 'logs':
            schema = self.log_schema
        else:
            raise ValueError(f"Unable to determine schema for table {table_id}")

        job_config = bigquery.LoadJobConfig(
            schema=schema,
            write_disposition=bigquery.WriteDisposition.WRITE_APPEND
        )

        # Process DataFrame in chunks to handle large datasets
        total_rows = len(df)
        for i in range(0, total_rows, chunk_size):
            chunk_df = df.iloc[i:i + chunk_size]
            
            job = self.client.load_table_from_dataframe(
                chunk_df,
                table_ref,
                job_config=job_config
            )
            
            job.result()  # Wait for the job to complete
            
            logger.info(f"Loaded rows {i} to {min(i + chunk_size, total_rows)}")
        
        logger.info(f"Successfully loaded {total_rows} total rows")

    def query_table(self, query: str) -> pd.DataFrame:
        """
        Execute a query and return results as DataFrame
        
        Args:
            query (str): SQL query to execute
            
        Returns:
            pd.DataFrame: Query results
        """
        return self.client.query(query).to_dataframe()
