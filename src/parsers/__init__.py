from .blocks import (
    ArbitrumBlockParser,
    EthereumBlockParser,
    ZKsyncBlockParser
)
from .logs import (
    ArbitrumLogParser,
    EthereumLogParser,
    ZKsyncLogParser
)
from .transactions import (
    ArbitrumTransactionParser,
    EthereumTransactionParser,
    ZKsyncTransactionParser
)
from rpc_types import (
    ArbitrumBlock,
    EthereumBlock,
    ZKsyncBlock,
    ArbitrumLog,
    EthereumLog,
    ZKsyncLog,
    ArbitrumTransaction,
    EthereumTransaction,
    ZKsyncTransaction
)

# Mapping to connect types with their parsers
BLOCK_PARSERS = {
    ArbitrumBlock: ArbitrumBlockParser,
    EthereumBlock: EthereumBlockParser,
    ZKsyncBlock: ZKsyncBlockParser,
}

LOG_PARSERS = {
    ArbitrumLog: ArbitrumLogParser,
    EthereumLog: EthereumLogParser,
    ZKsyncLog: ZKsyncLogParser,
}

TRANSACTION_PARSERS = {
    ArbitrumTransaction: ArbitrumTransactionParser,
    EthereumTransaction: EthereumTransactionParser,
    ZKsyncTransaction: ZKsyncTransactionParser,
}