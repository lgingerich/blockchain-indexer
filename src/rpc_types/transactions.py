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
        "arbitrary_types_allowed": True
    }
    
    address: str
    storageKeys: List[HexStr]

class BaseTransaction(BaseModel):
    model_config = {
        "arbitrary_types_allowed": True
    }
    
    blockHash: HexStr
    blockNumber: BlockNumber
    chainId: Optional[ChainId] = None
    from_address: str
    gas: Wei
    gasPrice: Wei
    hash: HexStr
    input: HexStr
    nonce: int
    r: Optional[HexStr] = None
    s: Optional[HexStr] = None
    to_address: str
    transactionIndex: int
    type: int
    v: Optional[int] = None
    value: Wei

# Same as BaseTransaction — keep here for clarity and completeness
class ArbitrumTransaction(BaseTransaction):
    pass

class EthereumTransaction(BaseTransaction):
    accessList: Optional[List[AccessListEntry]] = []
    blobVersionedHashes: Optional[List[HexStr]] = []
    maxFeePerBlobGas: Optional[Wei] = None # TO DO: Why should I use the Wei type?
    maxFeePerGas: Optional[Wei] = None
    maxPriorityFeePerGas: Optional[Wei] = None
    yParity: Optional[int] = None

class ZKsyncTransaction(BaseTransaction):
    l1BatchNumber: Optional[int] = None
    l1BatchTxIndex: Optional[int] = None
    maxFeePerGas: Wei
    maxPriorityFeePerGas: Wei
