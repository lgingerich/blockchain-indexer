from typing import List, Optional
from eth_typing import (
    BlockNumber,
    HexStr,
)
from web3.types import Wei
from pydantic import BaseModel

class Withdrawal(BaseModel):
    address: str
    amount: int
    index: int
    validatorIndex: int

class BaseBlock(BaseModel):
    model_config = {
        "arbitrary_types_allowed": True
    }
    
    baseFeePerGas: Optional[Wei] = None
    difficulty: int
    extraData: Optional[HexStr] = None
    gasLimit: Wei
    gasUsed: Wei
    hash: HexStr
    logsBloom: HexStr
    miner: str
    mixHash: HexStr
    nonce: HexStr
    number: BlockNumber
    parentHash: HexStr
    receiptsRoot: HexStr
    sha3Uncles: HexStr
    size: int
    stateRoot: HexStr
    timestamp: int
    totalDifficulty: int
    transactions: List[HexStr] = []
    transactionsRoot: HexStr
    uncles: List[HexStr] = []

class ArbitrumBlock(BaseBlock):
    l1BlockNumber: int
    sendCount: Optional[int] = None
    sendRoot: Optional[HexStr] = None

class EthereumBlock(BaseBlock):
    blobGasUsed: Optional[int] = None
    excessBlobGas: Optional[int] = None
    parentBeaconBlockRoot: Optional[HexStr] = None
    withdrawals: Optional[List[Withdrawal]] = []
    withdrawalsRoot: Optional[HexStr] = None

class ZKsyncBlock(BaseBlock):
    l1BatchNumber: Optional[int] = None
    l1BatchTimestamp: Optional[int] = None
    sealFields: List[HexStr] = []