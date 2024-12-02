from enum import Enum
from typing import Type
from .blocks import (
    BaseBlock,
    ArbitrumBlock,
    EthereumBlock,
    ZKsyncBlock,
)
from .logs import (
    BaseLog,
    ArbitrumLog,
    EthereumLog,
    ZKsyncLog
)
from .transactions import (
    BaseTransaction,
    ArbitrumTransaction,
    EthereumTransaction,
    ZKsyncTransaction
)

# Add new chains here. Applicable for all chains!
class ChainType(Enum):
    ARBITRUM = "arbitrum"
    CRONOS_ZKEVM = "cronos_zkevm"
    ETHEREUM = "ethereum"
    ZERO = "zero"
    ZKSYNC = "zksync"
    ZKSYNC_SEPOLIA = "zksync_sepolia"

Block = ArbitrumBlock | EthereumBlock | ZKsyncBlock
Log = ArbitrumLog | EthereumLog | ZKsyncLog
Transaction = ArbitrumTransaction | EthereumTransaction | ZKsyncTransaction

# Mapping of ChainType to Block class
# Add new chains here. Applicable for all chains!
# If the new chain does not fit an existing class, add a new class and add it to the mapping.
BLOCK_TYPE_MAPPING: dict[ChainType, Type[BaseBlock]] = {
    ChainType.ARBITRUM: ArbitrumBlock,
    ChainType.CRONOS_ZKEVM: ZKsyncBlock,
    ChainType.ETHEREUM: EthereumBlock,
    ChainType.ZERO: ZKsyncBlock,
    ChainType.ZKSYNC: ZKsyncBlock,
    ChainType.ZKSYNC_SEPOLIA: ZKsyncBlock,
}

# Mapping of ChainType to Log class
# Add new chains here. Applicable for all chains!
# If the new chain does not fit an existing class, add a new class and add it to the mapping.
LOG_TYPE_MAPPING: dict[ChainType, Type[BaseLog]] = {
    ChainType.ARBITRUM: ArbitrumLog,
    ChainType.CRONOS_ZKEVM: ZKsyncLog,
    ChainType.ETHEREUM: EthereumLog,
    ChainType.ZERO: ZKsyncLog,
    ChainType.ZKSYNC: ZKsyncLog,
    ChainType.ZKSYNC_SEPOLIA: ZKsyncLog,
}

# Mapping of ChainType to Transaction class
# Add new chains here. Applicable for all chains!
# If the new chain does not fit an existing class, add a new class and add it to the mapping.
TRANSACTION_TYPE_MAPPING: dict[ChainType, Type[BaseTransaction]] = {
    ChainType.ARBITRUM: ArbitrumTransaction,
    ChainType.CRONOS_ZKEVM: ZKsyncTransaction,
    ChainType.ETHEREUM: EthereumTransaction,
    ChainType.ZERO: ZKsyncTransaction,
    ChainType.ZKSYNC: ZKsyncTransaction,
    ChainType.ZKSYNC_SEPOLIA: ZKsyncTransaction,
}
