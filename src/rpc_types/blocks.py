from typing import List, Optional, TypedDict
from eth_typing import (
    BlockNumber,
    Address,
    HexStr,
)
from hexbytes import HexBytes
from web3.types import Wei

class Withdrawal(TypedDict):
    address: str
    amount: int
    index: int
    validatorIndex: int

class BaseBlock(TypedDict):
    baseFeePerGas: Optional[Wei]
    difficulty: int
    extraData: HexBytes
    gasLimit: Wei
    gasUsed: Wei
    hash: HexBytes
    logsBloom: HexBytes
    miner: Address
    mixHash: HexBytes
    nonce: HexBytes
    number: BlockNumber
    parentHash: HexBytes
    receiptsRoot: HexBytes
    sha3Uncles: HexBytes
    size: int
    stateRoot: HexBytes
    timestamp: int
    totalDifficulty: int
    transactions: List[HexBytes]
    transactionsRoot: HexBytes
    uncles: List[HexBytes]

class ArbitrumBlock(BaseBlock):
    l1BlockNumber: int
    sendCount: Optional[int]
    sendRoot: Optional[HexStr]

class EthereumBlock(BaseBlock):
    blobGasUsed: Optional[int]
    excessBlobGas: Optional[int]
    parentBeaconBlockRoot: Optional[HexBytes]
    withdrawals: Optional[List[Withdrawal]] # TO DO: This should become it's own data set
    withdrawalsRoot: Optional[HexBytes]

class ZKsyncBlock(BaseBlock):
    l1BatchNumber: Optional[int]
    l1BatchTimestamp: Optional[int]
    sealFields: List[HexBytes]