from enum import Enum
from typing import Type, List
from .base import BaseDataManager
from .bigquery import BigQueryDataManager
# Future imports for other data managers:
# from .postgres import PostgresDataManager
# from .snowflake import SnowflakeDataManager
# from .cloud_storage import CloudStorageDataManager

class StorageType(Enum):
    BIGQUERY = "bigquery"
    # POSTGRES = "postgres"
    # SNOWFLAKE = "snowflake"
    # CLOUD_STORAGE = "cloud_storage"

class DataManagerFactory:
    _managers = {
        StorageType.BIGQUERY: BigQueryDataManager,
        # StorageType.POSTGRES: PostgresDataManager,
        # StorageType.SNOWFLAKE: SnowflakeDataManager,
        # StorageType.CLOUD_STORAGE: CloudStorageDataManager,
    }
    
    @classmethod
    def get_manager(cls, storage_type: str, chain_name: str, config: dict, active_datasets: List[str] | None = None) -> BaseDataManager:
        """
        Factory method to get the appropriate data manager instance
        
        Args:
            storage_type (str): Type of storage from config
            chain_name (str): Name of the chain
            config (dict): Storage-specific configuration
            active_datasets (List[str]): List of active datasets to manage
        Returns:
            BaseDataManager: Instance of the appropriate data manager
        """
        try:
            storage_enum = StorageType(storage_type.lower())
            manager_class = cls._managers.get(storage_enum)
            if not manager_class:
                raise ValueError(f"Unsupported storage type: {storage_type}")
            
            return manager_class(chain_name, location=config.location, active_datasets=active_datasets)
        except ValueError as e:
            raise ValueError(f"Invalid storage type: {storage_type}. Supported types: {[t.value for t in StorageType]}")

# For backwards compatibility and easier imports
def get_data_manager(storage_type: str, chain_name: str, config: dict, active_datasets: List[str] | None = None) -> BaseDataManager:
    return DataManagerFactory.get_manager(storage_type, chain_name, config, active_datasets)
