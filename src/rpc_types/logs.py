from typing import List, Optional
from eth_typing import (
    BlockNumber,
    HexStr,
)
from pydantic import BaseModel

class BaseLog(BaseModel):
    model_config = {
        "arbitrary_types_allowed": False
    }
    
    address: str
    block_hash: HexStr
    block_number: BlockNumber
    data: HexStr
    log_index: int
    removed: bool
    topics: List[HexStr]
    transaction_hash: HexStr
    transaction_index: int

# Same as BaseTransaction — keep here for clarity and completeness
# Note: Arbitrum does not return blockTimestamp in logs
class ArbitrumLog(BaseLog):
    pass

class EthereumLog(BaseLog):
    block_timestamp: Optional[int] = None

class ZKsyncLog(BaseLog):
    block_timestamp: Optional[int] = None
    # block_time: str
    # block_date: str
    l1_batch_number: Optional[int] = None
    log_type: Optional[str] = None
    transaction_log_index: Optional[int] = None