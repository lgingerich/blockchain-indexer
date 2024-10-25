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
    CRONOS_ZKEVM = "cronos-zkevm"
    ETHEREUM = "ethereum"
    ZKSYNC = "zksync"

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
    ChainType.ZKSYNC: ZKsyncBlock,
}

# Mapping of ChainType to Log class
# Add new chains here. Applicable for all chains!
# If the new chain does not fit an existing class, add a new class and add it to the mapping.
LOG_TYPE_MAPPING: dict[ChainType, Type[BaseLog]] = {
    ChainType.ARBITRUM: ArbitrumLog,
    ChainType.CRONOS_ZKEVM: ZKsyncLog,
    ChainType.ETHEREUM: EthereumLog,
    ChainType.ZKSYNC: ZKsyncLog,
}

# Mapping of ChainType to Transaction class
# Add new chains here. Applicable for all chains!
# If the new chain does not fit an existing class, add a new class and add it to the mapping.
TRANSACTION_TYPE_MAPPING: dict[ChainType, Type[BaseTransaction]] = {
    ChainType.ARBITRUM: ArbitrumTransaction,
    ChainType.CRONOS_ZKEVM: ZKsyncTransaction,
    ChainType.ETHEREUM: EthereumTransaction,
    ChainType.ZKSYNC: ZKsyncTransaction,
}
