from loguru import logger
from web3 import AsyncWeb3, AsyncHTTPProvider
from eth_typing import BlockNumber
from rpc_types import (
    ChainType,
    Block,
    Transaction,
    BLOCK_TYPE_MAPPING,
    TRANSACTION_TYPE_MAPPING
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

    async def get_block(self, block_number: BlockNumber) -> dict:
        logger.info(f"Fetching block with number: {block_number}")
        raw_block = await self.w3.eth.get_block(block_number, full_transactions=True)
        return dict(raw_block)

    async def process_block(self, raw_block: dict) -> Block:
        block_class = BLOCK_TYPE_MAPPING.get(self.chain_type)
        if block_class is None:
            logger.error(f"Unsupported chain type: {self.chain_type}")
            raise ValueError(f"Unsupported chain type: {self.chain_type}")
        
        # Create a copy of the block data before modifying it
        block_data = raw_block.copy()
        
        # Extract only transaction hashes from full transaction data
        block_data['transactions'] = [tx['hash'] for tx in block_data['transactions']]
        
        return block_class(**block_data)
        
    async def process_transactions(self, raw_transactions: list) -> list[Transaction]:
        # First rename fields for all transactions
        transformed_transactions = []
        for tx in raw_transactions:
            tx_dict = dict(tx)
            tx_dict['from_address'] = tx_dict.pop('from')
            tx_dict['to_address'] = tx_dict.pop('to')
            transformed_transactions.append(tx_dict)

        # Then apply type mapping to the renamed transactions
        transaction_class = TRANSACTION_TYPE_MAPPING.get(self.chain_type)
        if transaction_class is None:
            logger.error(f"Unsupported chain type for transactions: {self.chain_type}")
            raise ValueError(f"Unsupported chain type for transactions: {self.chain_type}")

        return [transaction_class(**tx) for tx in transformed_transactions]
