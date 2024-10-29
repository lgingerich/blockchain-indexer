from loguru import logger
from web3 import AsyncWeb3, AsyncHTTPProvider
from typing import Dict, Any, cast, List
from utils import async_retry
from web3.exceptions import Web3Exception, BlockNotFound
from rpc_types import (
    ChainType,
    Block,
    BLOCK_TYPE_MAPPING,
    Transaction,
    TRANSACTION_TYPE_MAPPING,
    Log,
    LOG_TYPE_MAPPING
)
from parsers import (
    BLOCK_PARSERS, 
    TRANSACTION_PARSERS,
    LOG_PARSERS
)

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
        except BlockNotFound: # TO DO: Should this raise an error?
            logger.warning(f"Block {block_number} not found") 
            return None
        except Web3Exception as e:
            logger.error(f"Failed to get block {block_number}: {str(e)}")
            raise

    @async_retry(retries=5, base_delay=1, exponential_backoff=True, jitter=True)
    async def get_logs(self, block_number: int) -> List[dict] | None:
        try:
            raw_logs = await self.w3.eth.get_logs({'fromBlock': block_number, 'toBlock': block_number})
            if not raw_logs:
                logger.info(f"No logs found for block number: {block_number}")
            return raw_logs
        except Web3Exception as e:
            logger.error(f"Failed to get logs for block {block_number}: {str(e)}")
            raise

    @async_retry(retries=5, base_delay=1, exponential_backoff=True, jitter=True)
    async def parse_block(self, raw_block: int) -> Block | None:
        try:
            if not raw_block:
                logger.warning("No block data to parse")
                return None
            logger.info(f"Parsing block data")
            block_class = BLOCK_TYPE_MAPPING[self.chain_type]
            parser_class = BLOCK_PARSERS[block_class]
            parsed_data = parser_class.parse_raw(raw_block)
            return cast(Block, block_class(**parsed_data))
        except Exception as e:
            logger.error(f"Failed to parse block data: {str(e)}")
            raise

    @async_retry(retries=5, base_delay=1, exponential_backoff=True, jitter=True)
    async def parse_transactions(self, raw_transactions: List[Dict[str, Any]]) -> List[Transaction] | None:
        try:
            if not raw_transactions:
                logger.info("No transactions to parse")
                return []
            logger.info(f"Parsing transaction data")
            transaction_class = TRANSACTION_TYPE_MAPPING[self.chain_type]
            parser_class = TRANSACTION_PARSERS[transaction_class]
            parsed_data = [parser_class.parse_raw(raw_tx) for raw_tx in raw_transactions]
            return cast(List[Transaction], [transaction_class(**parsed_tx) for parsed_tx in parsed_data])
        except Exception as e:
            logger.error(f"Failed to parse transaction data: {str(e)}")
            raise

    @async_retry(retries=5, base_delay=1, exponential_backoff=True, jitter=True)
    async def parse_logs(self, raw_logs: List[dict]) -> List[Log] | None:
        try:
            if not raw_logs:
                logger.info("No logs to parse")
                return []
            logger.info(f"Parsing log data")
            log_class = LOG_TYPE_MAPPING[self.chain_type]
            parser_class = LOG_PARSERS[log_class]
            parsed_data = [parser_class.parse_raw(raw_log) for raw_log in raw_logs]
            return cast(List[Log], [log_class(**parsed_log) for parsed_log in parsed_data])
        except Exception as e:
            logger.error(f"Failed to parse log data: {str(e)}")
            raise