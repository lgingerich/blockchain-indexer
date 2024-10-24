from typing import List, Optional, TypedDict
from eth_typing import (
    Address,
    BlockNumber,
    ChainId,
    Hash32,
    HexStr
)
from web3.types import Wei

class AccessListEntry(TypedDict):
    address: Address
    storageKeys: List[HexStr]

class BaseTransaction(TypedDict):
    blockHash: Hash32
    blockNumber: BlockNumber
    chainId: Optional[ChainId]
    from_address: Address
    gas: Wei
    gasPrice: Wei
    hash: Hash32
    input: HexStr
    nonce: int
    r: Optional[HexStr]
    s: Optional[HexStr]
    to_address: Address
    transactionIndex: int
    type: int
    v: Optional[int]
    value: Wei

# Same as BaseTransaction — keep here for clarity and completeness
class ArbitrumTransaction(BaseTransaction):
    pass

class EthereumTransaction(BaseTransaction):
    accessList: Optional[List[AccessListEntry]]
    blobVersionedHashes: Optional[List[HexStr]]
    maxFeePerBlobGas: Optional[Wei] # TO DO: Why should I use the Wei type?
    maxFeePerGas: Optional[Wei]
    maxPriorityFeePerGas: Optional[Wei]
    yParity: int

class ZKsyncTransaction(BaseTransaction):
    l1BatchNumber: HexStr
    l1BatchTxIndex: HexStr
    maxFeePerGas: Wei
    maxPriorityFeePerGas: Wei
