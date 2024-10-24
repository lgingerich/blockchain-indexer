from enum import Enum
from typing import TypedDict, List, Optional, Type
from eth_typing import (
    BlockNumber,
    Hash32,
    Address,
    HexStr,
)
from web3.types import Wei


# TO DO: these fields have changed with blobs now. need to check earliest and latest blocks


class ChainType(Enum):
    ETHEREUM = "ethereum"
    ARBITRUM = "arbitrum"
    OPTIMISM = "optimism"
    ZKSYNC = "zksync"

class BaseBlock(TypedDict):
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
    transactions: List[Hash32]
    transactionsRoot: Hash32
    uncles: List[Hash32]

# Exactly the same as BaseBlock. Keep in for clarity.
class EthereumBlock(BaseBlock):
    pass

class ArbitrumBlock(BaseBlock):
    baseFeePerGas: Wei
    l1BlockNumber: Optional[int]
    sendCount: Optional[int]
    sendRoot: Optional[Hash32]
    totalDifficulty: int

class OptimismBlock(BaseBlock):
    baseFeePerGas: Wei
    totalDifficulty: int

class ZKsyncBlock(BaseBlock):
    baseFeePerGas: Wei
    l1BatchNumber: Optional[int]
    l1BatchTimestamp: Optional[int]
    sealFields: List[HexStr]
    totalDifficulty: int

Block = EthereumBlock | ArbitrumBlock | OptimismBlock | ZKsyncBlock


# Mapping of ChainType to Block class
BLOCK_TYPE_MAPPING: dict[ChainType, Type[BaseBlock]] = {
    ChainType.ETHEREUM: EthereumBlock,
    ChainType.ARBITRUM: ArbitrumBlock,
    ChainType.OPTIMISM: OptimismBlock,
    ChainType.ZKSYNC: ZKsyncBlock,
}