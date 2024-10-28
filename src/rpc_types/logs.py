from typing import List, Optional
from eth_typing import (
    BlockNumber,
    HexStr,
)
from pydantic import BaseModel

class BaseLog(BaseModel):
    model_config = {
        "arbitrary_types_allowed": True
    }
    
    address: str
    blockHash: HexStr
    blockNumber: BlockNumber
    data: HexStr
    logIndex: int
    removed: bool
    topics: List[HexStr]
    transactionHash: HexStr
    transactionIndex: int

# Same as BaseTransaction — keep here for clarity and completeness
# Note: Arbitrum does not return blockTimestamp in logs
class ArbitrumLog(BaseLog):
    pass

class EthereumLog(BaseLog):
    blockTimestamp: Optional[int] = None

class ZKsyncLog(BaseLog):
    blockTimestamp: Optional[int] = None
    l1BatchNumber: Optional[int] = None
    logType: str
    transactionLogIndex: Optional[int] = None