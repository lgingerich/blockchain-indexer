from pydantic import BaseModel
from datetime import datetime, date
from typing import List, Optional


class AccessListEntry(BaseModel):
    model_config = {
        "arbitrary_types_allowed": False,
        # "validate_all": False # Temporarily disable validation
    }
    
    address: str
    storage_keys: List[str]

class BaseTransaction(BaseModel):
    model_config = {
        "arbitrary_types_allowed": False,
        # "validate_all": False # Temporarily disable validation
    }
    
    # Fields from get_block transaction data
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

    # Fields from get_transaction_receipt
    status: int
    cumulative_gas_used: int
    effective_gas_price: int
    gas_used: int
    logs_bloom: str # check if this is always the same as BaseBlock.logs_bloom and delete if so
    contract_address: Optional[str] = None # is this needed? is this a duplicate?

# Same as BaseTransaction — keep here for clarity and completeness
class ArbitrumTransaction(BaseTransaction):
    # Fields from get_transaction_receipt
    blob_gas_used: Optional[int] = None # check if these 3 are actually optional
    l1_block_number: Optional[int] = None
    gas_used_for_l1: Optional[int] = None

class EthereumTransaction(BaseTransaction):
     # Fields from get_block transaction data
    access_list: Optional[List[AccessListEntry]] = []
    blob_versioned_hashes: Optional[List[str]] = []
    max_fee_per_blob_gas: Optional[int] = None
    max_fee_per_gas: Optional[int] = None
    max_priority_fee_per_gas: Optional[int] = None
    y_parity: Optional[int] = None

class ZKsyncTransaction(BaseTransaction):
    # Fields from get_block transaction data
    l1_batch_number: Optional[int] = None
    l1_batch_tx_index: Optional[int] = None
    max_fee_per_gas: int
    max_priority_fee_per_gas: int

    # Fields from get_transaction_receipt
    root: str
