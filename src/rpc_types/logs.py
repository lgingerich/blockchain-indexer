from typing import List, Optional
from pydantic import BaseModel

class BaseLog(BaseModel):
    model_config = {
        "arbitrary_types_allowed": False
    }
    
    address: str
    block_hash: str
    block_number: int
    data: str
    log_index: int
    removed: bool
    topics: List[str]
    transaction_hash: str
    transaction_index: int

# Same as BaseTransaction — keep here for clarity and completeness
# Note: Arbitrum does not return blockTimestamp in logs
class ArbitrumLog(BaseLog):
    pass

class EthereumLog(BaseLog):
    block_timestamp: Optional[int] = None

class ZKsyncLog(BaseLog):
    block_timestamp: Optional[int] = None
    l1_batch_number: Optional[int] = None
    log_type: Optional[str] = None
    transaction_log_index: Optional[int] = None