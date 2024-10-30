from pydantic import BaseModel
from datetime import datetime, date
from typing import List, Optional


class AccessListEntry(BaseModel):
    model_config = {
        "arbitrary_types_allowed": False
    }
    
    address: str
    storage_keys: List[str]

class BaseTransaction(BaseModel):
    model_config = {
        "arbitrary_types_allowed": False
    }
    
    block_hash: str
    block_number: int
    block_time: datetime
    block_date: date
    chain_id: Optional[int] = None
    from_address: str
    gas: int
    gas_price: int
    hash: str
    input: str
    nonce: int
    r: Optional[str] = None
    s: Optional[str] = None
    to_address: str
    transaction_index: int
    type: int
    v: Optional[int] = None
    value: str

# Same as BaseTransaction — keep here for clarity and completeness
class ArbitrumTransaction(BaseTransaction):
    pass

class EthereumTransaction(BaseTransaction):
    access_list: Optional[List[AccessListEntry]] = []
    blob_versioned_hashes: Optional[List[str]] = []
    max_fee_per_blob_gas: Optional[int] = None
    max_fee_per_gas: Optional[int] = None
    max_priority_fee_per_gas: Optional[int] = None
    y_parity: Optional[int] = None

class ZKsyncTransaction(BaseTransaction):
    l1_batch_number: Optional[int] = None
    l1_batch_tx_index: Optional[int] = None
    max_fee_per_gas: int
    max_priority_fee_per_gas: int
