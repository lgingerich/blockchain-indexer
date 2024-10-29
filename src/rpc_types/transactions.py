from typing import List, Optional
from eth_typing import (
    BlockNumber,
    ChainId,
    HexStr
)
from web3.types import Wei
from pydantic import BaseModel

class AccessListEntry(BaseModel):
    model_config = {
        "arbitrary_types_allowed": False
    }
    
    address: str
    storage_keys: List[HexStr]

class BaseTransaction(BaseModel):
    model_config = {
        "arbitrary_types_allowed": False
    }
    
    block_hash: HexStr
    block_number: BlockNumber
    chain_id: Optional[ChainId] = None
    from_address: str
    gas: Wei
    gas_price: Wei
    hash: HexStr
    input: HexStr
    nonce: int
    r: Optional[HexStr] = None
    s: Optional[HexStr] = None
    to_address: str
    transaction_index: int
    type: int
    v: Optional[int] = None
    value: Wei

# Same as BaseTransaction — keep here for clarity and completeness
class ArbitrumTransaction(BaseTransaction):
    pass

class EthereumTransaction(BaseTransaction):
    access_list: Optional[List[AccessListEntry]] = []
    blob_versioned_hashes: Optional[List[HexStr]] = []
    max_fee_per_blob_gas: Optional[Wei] = None # TO DO: Why should I use the Wei type?
    max_fee_per_gas: Optional[Wei] = None
    max_priority_fee_per_gas: Optional[Wei] = None
    y_parity: Optional[int] = None

class ZKsyncTransaction(BaseTransaction):
    l1_batch_number: Optional[int] = None
    l1_batch_tx_index: Optional[int] = None
    max_fee_per_gas: Wei
    max_priority_fee_per_gas: Wei
