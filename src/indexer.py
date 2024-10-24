import asyncio
from loguru import logger
from web3 import AsyncWeb3, AsyncHTTPProvider
from eth_typing import BlockNumber
from rpc_types import Block, ChainType, BLOCK_TYPE_MAPPING

class EVMIndexer:
    def __init__(self, rpc_url: str, chain_type: ChainType):
        self.w3 = AsyncWeb3(AsyncHTTPProvider(rpc_url))
        self.chain_type = chain_type

    async def get_block_number(self) -> BlockNumber:
        return await self.w3.eth.get_block_number()

    async def get_block(self, block_number: BlockNumber) -> Block:
        raw_block = await self.w3.eth.get_block(block_number)
        block_class = BLOCK_TYPE_MAPPING.get(self.chain_type)
        if block_class is None:
            raise ValueError(f"Unsupported chain type: {self.chain_type}")
        return block_class(**raw_block)
