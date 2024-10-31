from dataclasses import dataclass
from loguru import logger
from typing import Dict, Any, cast, List, Optional
from web3 import AsyncWeb3, AsyncHTTPProvider
from web3.exceptions import Web3Exception, BlockNotFound, TransactionNotFound

from parsers import BLOCK_PARSERS, TRANSACTION_PARSERS, LOG_PARSERS
from rpc_types import (
    ChainType,
    Block,
    Transaction,
    Log,
    BLOCK_TYPE_MAPPING,
    TRANSACTION_TYPE_MAPPING,
    LOG_TYPE_MAPPING
)
from utils import async_retry

@dataclass
class BlockData:
    block: Block
    transactions: List[Transaction]
    logs: List[Log]

class EVMIndexer:
    def __init__(self, rpc_url: str, chain_type: ChainType) -> None:
        logger.info(f"Initializing EVMIndexer for chain {chain_type.value} with RPC URL: {rpc_url}")
        self.w3 = AsyncWeb3(AsyncHTTPProvider(rpc_url))
        self.chain_type = chain_type

    @async_retry(retries=5, base_delay=1, exponential_backoff=True, jitter=True)
    async def get_block_number(self) -> int:
        try:
            block_number = await self.w3.eth.get_block_number()
            logger.info(f"Retrieved block number: {block_number}")
            return block_number
        except Web3Exception as e:
            logger.error(f"Failed to get block number: {str(e)}")
            raise

    @async_retry(retries=5, base_delay=1, exponential_backoff=True, jitter=True)
    async def get_block(self, block_number: int) -> dict | None:
        try:
            logger.info(f"Fetching block with number: {block_number}")
            raw_block = await self.w3.eth.get_block(block_number, full_transactions=True)
            return raw_block
        except BlockNotFound:
            logger.warning(f"Block {block_number} not found")
            return None
        except Web3Exception as e:
            logger.error(f"Failed to get block {block_number}: {str(e)}")
            raise

    @async_retry(retries=5, base_delay=1, exponential_backoff=True, jitter=True)
    async def get_transaction_receipt(self, transaction_hash: str) -> dict | None:
        try:
            receipt = await self.w3.eth.get_transaction_receipt(transaction_hash)
            return receipt
        except TransactionNotFound:
            logger.warning(f"Transaction {transaction_hash} not found")
            return None
        except Web3Exception as e:
            logger.error(f"Failed to get receipt for transaction {transaction_hash}: {str(e)}")
            raise

    @async_retry(retries=5, base_delay=1, exponential_backoff=True, jitter=True)
    async def parse_block_data(
        self,
        timestamp: int,
        block: dict,
        receipts: List[dict]
    ) -> BlockData:
        """Parse block, transactions, and logs in a single pass"""
        try:
            # Parse block
            block_class = BLOCK_TYPE_MAPPING[self.chain_type]
            block_parser = BLOCK_PARSERS[block_class]
            parsed_block_data = block_parser.parse_raw(block)
            parsed_block = cast(Block, block_class(**parsed_block_data))

            # Parse transactions
            transaction_class = TRANSACTION_TYPE_MAPPING[self.chain_type]
            transaction_parser = TRANSACTION_PARSERS[transaction_class]

            # Parse logs
            log_class = LOG_TYPE_MAPPING[self.chain_type]
            log_parser = LOG_PARSERS[log_class]

            parsed_transactions = []
            parsed_logs = []

            # Process each transaction and its receipt together
            for tx, receipt in zip(block['transactions'], receipts):
                
                # Parse transaction with receipt data
                parsed_tx_data = transaction_parser.parse_raw(tx, timestamp, receipt)
                parsed_tx = cast(Transaction, transaction_class(**parsed_tx_data))
                parsed_transactions.append(parsed_tx)

                # Parse logs from this transaction's receipt
                for log in receipt['logs']:
                    parsed_log_data = log_parser.parse_raw(log, timestamp)
                    parsed_log = cast(Log, log_class(**parsed_log_data))
                    parsed_logs.append(parsed_log)

            return BlockData(
                block=parsed_block,
                transactions=parsed_transactions,
                logs=parsed_logs
            )

        except Exception as e:
            logger.error(f"Failed to parse block data: {str(e)}")
            raise
