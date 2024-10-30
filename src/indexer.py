from loguru import logger
from typing import Dict, Any, cast, List
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
    async def parse_transactions(self, block_timestamp: int, raw_transactions: List[Dict[str, Any]]) -> List[Transaction] | None:
        try:
            if not raw_transactions:
                logger.info("No transactions to parse")
                return []
            logger.info(f"Parsing transaction data")
            transaction_class = TRANSACTION_TYPE_MAPPING[self.chain_type]
            parser_class = TRANSACTION_PARSERS[transaction_class]
            parsed_data = [parser_class.parse_raw(raw_tx, block_timestamp) for raw_tx in raw_transactions]
            return cast(List[Transaction], [transaction_class(**parsed_tx) for parsed_tx in parsed_data])
        except Exception as e:
            logger.error(f"Failed to parse transaction data: {str(e)}")
            raise

    @async_retry(retries=5, base_delay=1, exponential_backoff=True, jitter=True)
    async def parse_logs(self, block_timestamp: int, raw_logs: List[dict]) -> List[Log] | None:
        try:
            if not raw_logs:
                logger.info("No logs to parse")
                return []
            logger.info(f"Parsing log data")
            log_class = LOG_TYPE_MAPPING[self.chain_type]
            parser_class = LOG_PARSERS[log_class]
            parsed_data = [parser_class.parse_raw(raw_log, block_timestamp) for raw_log in raw_logs]
            return cast(List[Log], [log_class(**parsed_log) for parsed_log in parsed_data])
        except Exception as e:
            logger.error(f"Failed to parse log data: {str(e)}")
            raise


    @async_retry(retries=5, base_delay=1, exponential_backoff=True, jitter=True)
    async def get_receipts(self, transaction_hash: str) -> None:
        try:
            receipt = await self.w3.eth.get_transaction_receipt(transaction_hash)
            return receipt
        except TransactionNotFound:
            logger.warning(f"Transaction {transaction_hash} not found")
            return None


    # web3.eth.get_transaction_receipt('0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060')  # not yet mined
    # Traceback # ... etc ...
    # TransactionNotFound: Transaction with hash: 0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060 not found.

    # # wait for it to be mined....
    # web3.eth.get_transaction_receipt('0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060')
    # AttributeDict({
    #     'blockHash': '0x4e3a3754410177e6937ef1f84bba68ea139e8d1a2258c5f85db9f1cd715a1bdd',
    #     'blockNumber': 46147,
    #     'contractAddress': None,
    #     'cumulativeGasUsed': 21000,
    #     'from': '0xA1E4380A3B1f749673E270229993eE55F35663b4',
    #     'gasUsed': 21000,
    #     'logs': [],
    #     'logsBloom': '0x000000000000000000000000000000000000000000000000...0000',
    #     'status': 1, # 0 or 1
    #     'to': '0x5DF9B87991262F6BA471F09758CDE1c0FC1De734',
    #     'transactionHash': '0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060',
    #     'transactionIndex': 0,
    # })


    # nansen only adds in cumulative_gas_used, gas_used, contract_address, status
        # does the contract_address go with the transaction or the log?