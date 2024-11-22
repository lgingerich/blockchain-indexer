import os
import pandas as pd
from typing import List
from loguru import logger
import pyarrow as pa
import pyarrow.parquet as pq

from .base import BaseDataManager
from data_types import (
    ChainType,
    BLOCK_TYPE_MAPPING,
    TRANSACTION_TYPE_MAPPING,
    LOG_TYPE_MAPPING
)

class ParquetDataManager(BaseDataManager):
    """
    A class to manage Parquet file operations for blockchain data
    """
    
    def __init__(self, chain_name: str, active_datasets: List[str] | None = None, **kwargs):
        """
        Initialize Parquet storage manager
        
        Args:
            chain_name (str): Name of the chain to work with
            active_datasets (List[str]): List of active datasets to manage
            **kwargs: Configuration parameters
                - data_dir (str): Base directory for storing parquet files (default: "data")
        """
        self.chain_name = chain_name
        self.active_datasets = active_datasets
        
        # Get data directory from kwargs with default
        data_dir = kwargs.get('data_dir', 'data')
        
        # Create base directory path
        self.base_path = os.path.join(data_dir, chain_name)
        os.makedirs(self.base_path, exist_ok=True)
        
        # Create dataset directories if they don't exist
        for dataset in self.active_datasets:
            self.create_dataset(dataset)

    def create_dataset(self, dataset_id: str, **kwargs) -> None:
        """Creates a directory for the dataset if it doesn't exist"""
        dataset_path = os.path.join(self.base_path, dataset_id)
        os.makedirs(dataset_path, exist_ok=True)
        logger.info(f"Ensured dataset directory exists: {dataset_path}")

    def create_table(self, table_id: str, schema: List) -> None:
        """
        For Parquet, we don't need to pre-create tables as they're created when data is written
        """
        pass

    def load_table(self, df: pd.DataFrame, table_id: str, if_exists: str = 'append', **kwargs) -> None:
        """
        Load DataFrame into a Parquet file
        
        Args:
            df (pd.DataFrame): DataFrame to save
            table_id (str): Name of the table (used as directory name)
            if_exists (str): How to handle existing data:
                - 'fail': Raise error if file exists
                - 'replace': Overwrite existing file
                - 'append': Append to existing file
            **kwargs: Additional parameters
                - start_block (int): Start block number for this batch
                - end_block (int): End block number for this batch
        """
        if 'start_block' not in kwargs or 'end_block' not in kwargs:
            raise ValueError("start_block and end_block are required for parquet file naming")
        
        start_block = kwargs['start_block']
        end_block = kwargs['end_block']
        
        table_path = os.path.join(self.base_path, table_id)
        file_path = os.path.join(table_path, f"{table_id}_{start_block}_{end_block}.parquet")

        if os.path.exists(file_path):
            if if_exists == 'fail':
                raise ValueError(f"Table {table_id} already exists")
            elif if_exists == 'replace':
                df.to_parquet(file_path, index=False)
            elif if_exists == 'append':
                if os.path.exists(file_path):
                    existing_df = pd.read_parquet(file_path)
                    df = pd.concat([existing_df, df], ignore_index=True)
                df.to_parquet(file_path, index=False)
        else:
            df.to_parquet(file_path, index=False)

        logger.info(f"Saved {len(df)} rows to {file_path}")

    def query_table(self, query: str) -> pd.DataFrame:
        """
        Execute a query on the parquet files. 
        Note: This is a simplified implementation that loads the entire file.
        For production use, consider using DuckDB or similar for actual SQL queries.
        
        Args:
            query (str): SQL-like query (currently ignored, loads entire table)
            
        Returns:
            pd.DataFrame: Query results
        """
        # Extract table name from a simple SELECT query
        # This is a very basic implementation
        table_name = query.lower().split('from')[1].strip().split()[0].replace('`', '')
        table_name = table_name.split('.')[-1]  # Get last part if fully qualified
        
        file_path = os.path.join(self.base_path, table_name, "data.parquet")
        if not os.path.exists(file_path):
            return pd.DataFrame()
            
        return pd.read_parquet(file_path)

    def get_last_processed_block(self) -> int:
        """
        Get the lowest maximum block number across all active tables
        
        Returns:
            int: The lowest maximum block number across all active tables
        """
        min_block = None
        
        for table_id in self.active_datasets:
            table_path = os.path.join(self.base_path, table_id)
            if not os.path.exists(table_path):
                return 0
            
            try:
                # Get all parquet files for this table
                parquet_files = [f for f in os.listdir(table_path) if f.endswith('.parquet')]
                if not parquet_files:
                    return 0
                
                # Read block_number column from all files and get the max
                max_block = 0
                for file in parquet_files:
                    file_path = os.path.join(table_path, file)
                    try:
                        df = pd.read_parquet(file_path, columns=['block_number'])
                        if not df.empty:
                            max_block = max(max_block, int(df['block_number'].max()))
                    except Exception as e:
                        logger.warning(f"Failed to read block numbers from file {file}: {str(e)}")
                        continue
                
                if max_block == 0:
                    return 0
                    
                min_block = min(min_block, max_block) if min_block is not None else max_block
                
            except Exception as e:
                logger.warning(f"Failed to get max block number for table {table_id}: {str(e)}")
                return 0
                
        return min_block or 0
