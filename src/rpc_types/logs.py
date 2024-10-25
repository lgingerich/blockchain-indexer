from typing import List, Optional, TypedDict
from eth_typing import (
    BlockNumber,
    Hash32,
    Address,
    HexStr,
)
from web3.types import Wei

class BaseLog(TypedDict):
    pass

class ArbitrumLog(BaseLog):
    pass

class EthereumLog(BaseLog):
    pass

class ZKsyncLog(BaseLog):
    pass