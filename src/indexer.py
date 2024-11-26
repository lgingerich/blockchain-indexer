from dataclasses import dataclass
from loguru import logger
from typing import Dict, Any, cast, List, Optional
from web3 import AsyncWeb3, AsyncHTTPProvider
from web3.exceptions import Web3Exception, BlockNotFound, TransactionNotFound
from aiohttp import ClientSession, TCPConnector

from parsers import BLOCK_PARSERS, TRANSACTION_PARSERS, LOG_PARSERS
from data_types import (
    ChainType,
    Block,
    Transaction,
    Log,
    BLOCK_TYPE_MAPPING,
    TRANSACTION_TYPE_MAPPING,
    LOG_TYPE_MAPPING
)
from utils import async_retry
from metrics import (
    RPC_REQUESTS,
    RPC_ERRORS,
    RPC_LATENCY,
    BLOCK_PROCESSING_TIME,
)
import time

@dataclass
class BlockData:
    block: Block
    transactions: List[Transaction]
    logs: List[Log]

class EVMIndexer:
    def __init__(self, rpc_urls: List[str], chain_type: ChainType) -> None:
        logger.info(f"Available RPC URLs: {rpc_urls}")
        logger.info(f"Initializing EVMIndexer for chain {chain_type.value} with RPC URL: {rpc_urls[0]}")
        self.rpc_urls = rpc_urls
        self.current_rpc_index = 0
        self.w3 = AsyncWeb3(AsyncHTTPProvider(self.rpc_urls[0]))
        self.chain_type = chain_type
        
        # Initialize all metrics with initial values
        BLOCK_PROCESSING_TIME.labels(chain=self.chain_type.value).observe(0)
        
        # Initialize RPC metrics
        for method in ['get_block', 'get_block_number', 'get_transaction_receipt']:
            RPC_REQUESTS.labels(chain=self.chain_type.value, method=method).inc(0)
            RPC_ERRORS.labels(chain=self.chain_type.value, method=method).inc(0)
            RPC_LATENCY.labels(chain=self.chain_type.value, method=method).observe(0)

    def _rotate_rpc(self) -> bool:
        """Rotate to the next RPC URL in the list
        Returns:
            bool: True if there is another RPC to rotate to, False if we've tried all RPCs
        """
        if len(self.rpc_urls) <= 1:
            return False
            
        self.current_rpc_index = (self.current_rpc_index + 1) % len(self.rpc_urls)
        new_url = self.rpc_urls[self.current_rpc_index]
        logger.info(f"Switching to RPC URL: {new_url}")
        self.w3 = AsyncWeb3(AsyncHTTPProvider(new_url))
        return True

    @async_retry(retries=5, base_delay=2, exponential_backoff=True, jitter=True)
    async def get_block_number(self) -> int:
        start_time = time.time()
        try:
            block_number = await self.w3.eth.get_block_number()
            RPC_REQUESTS.labels(chain=self.chain_type.value, method='get_block_number').inc()
            RPC_LATENCY.labels(chain=self.chain_type.value, method='get_block_number').observe(time.time() - start_time)
            return block_number
        except Web3Exception as e:
            RPC_ERRORS.labels(chain=self.chain_type.value, method='get_block_number').inc()
            logger.error(f"Failed to get block number: {str(e)}")
            if self._rotate_rpc():
                return await self.get_block_number()
            raise
        except Exception as e:
            RPC_ERRORS.labels(chain=self.chain_type.value, method='get_block_number').inc()
            logger.error(f"Failed to get block number: {type(e).__name__}: {str(e)}")
            if self._rotate_rpc():
                return await self.get_block_number()
            raise

    @async_retry(retries=5, base_delay=2, exponential_backoff=True, jitter=True)
    async def get_block(self, block_number: int) -> dict | None:
        start_time = time.time()
        try:
            logger.info(f"Fetching block with number: {block_number}")
            raw_block = await self.w3.eth.get_block(block_number, full_transactions=True)
            RPC_REQUESTS.labels(chain=self.chain_type.value, method='get_block').inc()
            RPC_LATENCY.labels(chain=self.chain_type.value, method='get_block').observe(time.time() - start_time)
            return raw_block
        except BlockNotFound:
            RPC_ERRORS.labels(chain=self.chain_type.value, method='get_block').inc()
            logger.warning(f"Block {block_number} not found")
            if self._rotate_rpc():
                return await self.get_block(block_number)
            return None
        except Web3Exception as e:
            RPC_ERRORS.labels(chain=self.chain_type.value, method='get_block').inc()
            logger.error(f"Failed to get block {block_number}: {str(e)}")
            if self._rotate_rpc():
                return await self.get_block(block_number)
            raise
        except Exception as e:
            RPC_ERRORS.labels(chain=self.chain_type.value, method='get_block').inc()
            logger.error(f"Failed to get block {block_number}: {type(e).__name__}: {str(e)}")
            if self._rotate_rpc():
                return await self.get_block(block_number)
            raise

    @async_retry(retries=5, base_delay=2, exponential_backoff=True, jitter=True)
    async def get_transaction_receipt(self, transaction_hash: str) -> dict | None:
        start_time = time.time()
        try:
            receipt = await self.w3.eth.get_transaction_receipt(transaction_hash)
            RPC_REQUESTS.labels(chain=self.chain_type.value, method='get_transaction_receipt').inc()
            RPC_LATENCY.labels(chain=self.chain_type.value, method='get_transaction_receipt').observe(time.time() - start_time)
            return receipt
        except TransactionNotFound:
            RPC_ERRORS.labels(chain=self.chain_type.value, method='get_transaction_receipt').inc()
            logger.warning(f"Transaction {transaction_hash} not found")
            if self._rotate_rpc():
                return await self.get_transaction_receipt(transaction_hash)
            return None
        except Web3Exception as e:
            RPC_ERRORS.labels(chain=self.chain_type.value, method='get_transaction_receipt').inc()
            logger.error(f"Failed to get receipt for transaction {transaction_hash}: {str(e)}")
            if self._rotate_rpc():
                return await self.get_transaction_receipt(transaction_hash)
            raise
        except Exception as e:
            RPC_ERRORS.labels(chain=self.chain_type.value, method='get_transaction_receipt').inc()
            logger.error(f"Failed to get receipt for transaction {transaction_hash}: {type(e).__name__}: {str(e)}")
            if self._rotate_rpc():
                return await self.get_transaction_receipt(transaction_hash)
            raise

    @async_retry(retries=5, base_delay=2, exponential_backoff=True, jitter=True)
    async def parse_block_data(
        self,
        timestamp: int,
        block: dict,
        receipts: List[dict]
    ) -> BlockData:
        start_time = time.time()
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

            BLOCK_PROCESSING_TIME.labels(chain=self.chain_type.value).observe(time.time() - start_time)
            return BlockData(
                block=parsed_block,
                transactions=parsed_transactions,
                logs=parsed_logs
            )

        except Exception as e:
            logger.error(f"Failed to parse block data: {str(e)}")
            raise
