from abc import ABC, abstractmethod
import pandas as pd
from typing import List, Dict, Any

class BaseDataManager(ABC):
    """Abstract base class for all data managers"""
    
    @abstractmethod
    def __init__(self, chain_name: str, active_datasets: List[str] | None = None, **kwargs):
        """
        Initialize data manager
        
        Args:
            chain_name (str): Name of the chain to work with
            active_datasets (List[str] | None): List of active datasets to manage
            **kwargs: Implementation-specific configuration parameters
        """
        pass
    
    @abstractmethod
    def create_dataset(self, dataset_id: str, **kwargs) -> None:
        pass
    
    @abstractmethod
    def create_table(self, table_id: str, schema: List) -> None:
        pass
    
    @abstractmethod
    def load_table(self, 
        df: pd.DataFrame, 
        table_id: str, 
        if_exists: str = 'append',
        **kwargs
    ) -> None:
        """
        Load DataFrame into storage
        
        Args:
            df (pd.DataFrame): DataFrame to save
            table_id (str): Name of the table
            if_exists (str): How to handle existing data ('fail', 'replace', 'append')
            **kwargs: Implementation-specific parameters
        """
        pass
    
    @abstractmethod
    def query_table(self, query: str) -> pd.DataFrame:
        pass
    
    @abstractmethod
    def get_last_processed_block(self) -> int:
        pass