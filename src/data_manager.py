from google.cloud import bigquery
from google.oauth2 import service_account
import pandas as pd
from typing import Optional, Dict, List
from loguru import logger

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
        
        # Predefined schema for blockchain data
        self.schema = [
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
            bigquery.SchemaField('l1_batch_timestamp', 'INTEGER', mode='NULLABLE'),
            bigquery.SchemaField('seal_fields', 'STRING', mode='REPEATED')
        ]
        
        # Create dataset if it doesn't exist on client initialization
        self.create_dataset(self.dataset_id, location="US")

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
        except Exception:
            # Table doesn't exist, create new one
            pass
        
        # Create table with predefined schema
        table = bigquery.Table(table_ref, schema=self.schema)
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
        job_config = bigquery.LoadJobConfig(
            schema=self.schema,
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
