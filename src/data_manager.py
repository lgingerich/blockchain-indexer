from google.cloud import bigquery
from google.oauth2 import service_account
import pandas as pd
from typing import Optional, Dict, List
from loguru import logger
import google.api_core.exceptions

class BigQueryManager:
    """
    A class to manage BigQuery operations for blockchain data
    """
    
    def __init__(self, credentials_path: str, dataset_id: str):
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
        self.dataset_id = dataset_id
        
        # Add new block schema for ZKSync
        self.block_schema = [
            bigquery.SchemaField('base_fee_per_gas', 'INTEGER', mode='NULLABLE'),
            bigquery.SchemaField('difficulty', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('extra_data', 'STRING', mode='NULLABLE'),
            bigquery.SchemaField('gas_limit', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('gas_used', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('hash', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('logs_bloom', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('miner', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('mix_hash', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('nonce', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('number', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('parent_hash', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('receipts_root', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('sha3_uncles', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('size', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('state_root', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('block_time', 'TIMESTAMP', mode='REQUIRED'),
            bigquery.SchemaField('block_date', 'DATE', mode='REQUIRED'),
            bigquery.SchemaField('total_difficulty', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('transactions', 'STRING', mode='REPEATED'),
            bigquery.SchemaField('transactions_root', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('uncles', 'STRING', mode='REPEATED'),
            bigquery.SchemaField('l1_batch_number', 'INTEGER', mode='NULLABLE'),
            bigquery.SchemaField('l1_batch_time', 'TIMESTAMP', mode='NULLABLE'),
            bigquery.SchemaField('seal_fields', 'STRING', mode='REPEATED')
        ]
        
        # Add new transaction schema for ZKSync
        self.transaction_schema = [
            bigquery.SchemaField('block_hash', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('block_number', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('block_time', 'TIMESTAMP', mode='REQUIRED'),
            bigquery.SchemaField('block_date', 'DATE', mode='REQUIRED'),
            bigquery.SchemaField('chain_id', 'INTEGER', mode='NULLABLE'),
            bigquery.SchemaField('from_address', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('gas', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('gas_price', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('hash', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('input', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('nonce', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('r', 'STRING', mode='NULLABLE'),
            bigquery.SchemaField('s', 'STRING', mode='NULLABLE'),
            bigquery.SchemaField('to_address', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('transaction_index', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('type', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('v', 'INTEGER', mode='NULLABLE'),
            bigquery.SchemaField('value', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('l1_batch_number', 'INTEGER', mode='NULLABLE'),
            bigquery.SchemaField('l1_batch_tx_index', 'INTEGER', mode='NULLABLE'),
            bigquery.SchemaField('max_fee_per_gas', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('max_priority_fee_per_gas', 'INTEGER', mode='REQUIRED'),
        ]

        # Add new log schema for ZKSync
        self.log_schema = [
            bigquery.SchemaField('address', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('block_hash', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('block_number', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('block_time', 'TIMESTAMP', mode='REQUIRED'),
            bigquery.SchemaField('block_date', 'DATE', mode='REQUIRED'),
            bigquery.SchemaField('data', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('log_index', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('removed', 'BOOLEAN', mode='REQUIRED'),
            bigquery.SchemaField('topics', 'STRING', mode='REPEATED'),
            bigquery.SchemaField('transaction_hash', 'STRING', mode='REQUIRED'),
            bigquery.SchemaField('transaction_index', 'INTEGER', mode='REQUIRED'),
            bigquery.SchemaField('l1_batch_number', 'INTEGER', mode='NULLABLE'),
            bigquery.SchemaField('log_type', 'STRING', mode='NULLABLE'),
            bigquery.SchemaField('transaction_log_index', 'INTEGER', mode='NULLABLE')
        ]
        
        # Create dataset if it doesn't exist on client initialization
        self.create_dataset(self.dataset_id, location="US")
        
        # Create tables if they don't exist
        table_configs = {
            'blocks': self.block_schema,
            'transactions': self.transaction_schema,
            'logs': self.log_schema
        }
        
        for table_id, schema in table_configs.items():
            table_ref = self.client.dataset(self.dataset_id).table(table_id)
            try:
                self.client.get_table(table_ref)
                logger.info(f"Table {table_id} already exists")
            except google.api_core.exceptions.NotFound:
                table = bigquery.Table(table_ref, schema=schema)
                table = self.client.create_table(table)
                logger.info(f"Created table {table.project}.{table.dataset_id}.{table.table_id}")

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

    def create_and_load_table(self, 
        df: pd.DataFrame, 
        table_id: str, 
        if_exists: str = 'append',
        chunk_size: int = 10000) -> None:
        """
        Create a new table and load DataFrame data
        
        Args:
            df (pd.DataFrame): DataFrame containing the blockchain data
            table_id (str): ID for the new table
            if_exists (str): Action if table exists ('fail', 'replace', or 'append')
            chunk_size (int): Number of rows to load in each batch
        
        Raises:
            ValueError: If table exists and if_exists='fail'
        """
        table_ref = self.client.dataset(self.dataset_id).table(table_id)
        
        # Check if table exists
        try:
            existing_table = self.client.get_table(table_ref)
            if if_exists == 'fail':
                raise ValueError(f"Table {table_id} already exists")
            elif if_exists == 'replace':
                self.client.delete_table(table_ref)
            elif if_exists == 'append':
                self._load_dataframe(df, table_ref, chunk_size)
                return
        except google.api_core.exceptions.NotFound:
            # Table doesn't exist, continue to create new one
            logger.info(f"Table {table_id} does not exist, creating new table")
        except Exception as e:
            # Re-raise any other unexpected exceptions
            logger.error(f"Unexpected error while checking table {table_id}: {str(e)}")
            raise
        
        # Update to use the correct schema based on table type
        schema = None
        if table_id == 'blocks':
            schema = self.block_schema
        elif table_id == 'transactions':
            schema = self.transaction_schema
        elif table_id == 'logs':
            schema = self.log_schema
        else:
            raise ValueError(f"Unable to determine schema for table {table_id}")
        
        # Create table with appropriate schema
        table = bigquery.Table(table_ref, schema=schema)
        table = self.client.create_table(table)
        logger.info(f"Created table {table.project}.{table.dataset_id}.{table.table_id}")
        
        # Load data
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
