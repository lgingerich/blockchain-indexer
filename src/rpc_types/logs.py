from typing import List, TypedDict
from eth_typing import (
    BlockNumber,
    Address,
    HexStr,
)
from hexbytes import HexBytes

class BaseLog(TypedDict):
    address: Address
    blockHash: HexBytes
    blockNumber: BlockNumber
    data: HexBytes
    logIndex: int
    removed: bool
    topics: List[HexBytes]
    transactionHash: HexBytes
    transactionIndex: int

# Same as BaseTransaction — keep here for clarity and completeness
# Note: Arbitrum does not return blockTimestamp in logs
class ArbitrumLog(BaseLog):
    pass

class EthereumLog(BaseLog):
    blockTimestamp: int

class ZKsyncLog(BaseLog):
    blockTimestamp: int
    l1BatchNumber: int
    logType: str
    transactionLogIndex: int