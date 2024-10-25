from loguru import logger
from web3 import AsyncWeb3, AsyncHTTPProvider
from eth_typing import BlockNumber
from typing import Dict, Any, cast, List
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

    async def get_block_number(self) -> BlockNumber:
        block_number = await self.w3.eth.get_block_number()
        logger.info(f"Retrieved block number: {block_number}")
        return block_number

    async def get_block(self, block_number: BlockNumber) -> Dict[str, Any]:
        logger.info(f"Fetching block with number: {block_number}")
        raw_block = await self.w3.eth.get_block(block_number, full_transactions=True)
        return dict(raw_block)

    async def parse_block(self, raw_block: Dict[str, Any]) -> Block:
        logger.info(f"Parsing block data")
        block_class = BLOCK_TYPE_MAPPING[self.chain_type]
        parser_class = BLOCK_PARSERS[block_class]
        parsed_data = parser_class.parse_raw(raw_block)
        return cast(Block, block_class(parsed_data))
    
    async def parse_transactions(self, raw_transactions: List[Dict[str, Any]]) -> List[Transaction]:
        logger.info(f"Parsing transaction data")
        transaction_class = TRANSACTION_TYPE_MAPPING[self.chain_type]
        parser_class = TRANSACTION_PARSERS[transaction_class]
        parsed_data = [parser_class.parse_raw(raw_tx) for raw_tx in raw_transactions]
        return cast(List[Transaction], [transaction_class(parsed_tx) for parsed_tx in parsed_data])