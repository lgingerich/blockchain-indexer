from typing import List, Optional, TypedDict
from eth_typing import (
    BlockNumber,
    Hash32,
    Address,
    HexStr,
)
from web3.types import Wei

class Withdrawal(TypedDict):
    address: str
    amount: int
    index: int
    validatorIndex: int

class BaseBlock(TypedDict):
    baseFeePerGas: Optional[Wei]
    difficulty: int
    extraData: HexStr
    gasLimit: Wei
    gasUsed: Wei
    hash: Hash32
    logsBloom: HexStr
    miner: Address
    mixHash: Hash32
    nonce: HexStr
    number: BlockNumber
    parentHash: Hash32
    receiptsRoot: Hash32
    sha3Uncles: Hash32
    size: int
    stateRoot: Hash32
    timestamp: int
    totalDifficulty: int
    transactions: List[Hash32]
    transactionsRoot: Hash32
    uncles: List[Hash32]

class ArbitrumBlock(BaseBlock):
    l1BlockNumber: int
    sendCount: Optional[int]
    sendRoot: Optional[Hash32]

class EthereumBlock(BaseBlock):
    blobGasUsed: Optional[int]
    excessBlobGas: Optional[int]
    parentBeaconBlockRoot: Optional[Hash32]
    withdrawals: Optional[List[Withdrawal]] # TO DO: This should become it's own data set
    withdrawalsRoot: Optional[Hash32]

class ZKsyncBlock(BaseBlock):
    l1BatchNumber: Optional[int]
    l1BatchTimestamp: Optional[int]
    sealFields: List[HexStr]