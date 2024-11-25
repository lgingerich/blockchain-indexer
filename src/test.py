# from dataclasses import dataclass
# from loguru import logger
# from typing import Dict, Any, cast, List, Optional
from web3 import AsyncWeb3, AsyncHTTPProvider, Web3, HTTPProvider
from web3.exceptions import Web3Exception, BlockNotFound, TransactionNotFound
# from aiohttp import ClientSession, TCPConnector

# from parsers import BLOCK_PARSERS, TRANSACTION_PARSERS, LOG_PARSERS
# from rpc_types import (
#     ChainType,
#     Block,
#     Transaction,
#     Log,
#     BLOCK_TYPE_MAPPING,
#     TRANSACTION_TYPE_MAPPING,
#     LOG_TYPE_MAPPING
# )
# from utils import async_retry

# class EVMIndexer:
#     def __init__(self, rpc_url: str, chain_type: ChainType, max_connections: int = 100) -> None:
#         logger.info(f"Initializing EVMIndexer for chain {chain_type.value} with RPC URL: {rpc_url}")
#         # Create connection pool
#         self.session = ClientSession(
#             connector=TCPConnector(
#                 limit=max_connections,
#                 enable_cleanup_closed=True
#             )
#         )
#         self.w3 = AsyncWeb3(AsyncHTTPProvider(
#             rpc_url,
#             request_kwargs={'session': self.session}
#         ))
#         self.chain_type = chain_type

#     async def trace_replay_transaction(self, tx_hash: str) -> None:
#         trace = await self.w3.tracing.get_transaction_receipt(tx_hash)
#         logger.info(trace)


# from loguru import logger
# from web3 import Web3, HTTPProvider

# # Initialize web3 instance
# w3 = Web3(HTTPProvider('https://sepolia.era.zksync.dev'))

# def get_transaction_trace(tx_hash: str) -> list[dict] | None:
#     """Function used for retreiving trace to identify native ETH transfers."""
#     try:
#         res = w3.tracing.trace_transaction(tx_hash)
#         return res
#     except Exception as err:
#         logger.error(f"Error occurred while fetching transaction trace: {err}")

# # Call function with hash
# hash = '0x1b6cecdc9cd38d437967fed54f85823cc6ccce25890eda446aca80c20798c6fb'
# trace = get_transaction_trace(hash)
# print(trace)




# def get_transaction_trace(tx_hash: str) -> list[dict] | None:
#     """Function used for retreiving trace to identify native ETH transfers."""
#     try:
#         res = w3.tracing.trace_transaction(tx_hash)
#         return res
#     except Exception as err:
#         logger.error(f"Error occurred while fetching transaction trace: {err}")

# # Call function with hash
# hash = '0x1b6cecdc9cd38d437967fed54f85823cc6ccce25890eda446aca80c20798c6fb'
# trace = get_transaction_trace(hash)
# print(trace)


# from loguru import logger
# from web3 import Web3, HTTPProvider

# # Initialize web3 instance
# w3 = Web3(HTTPProvider('https://rpc.ankr.com/arbitrum'))


# def get_block_receipts(block_number: int):
#     try:
#         res = w3.eth.get_block_receipts(block_number)
#         return res
#     except Exception as err:
#         logger.error(f"Error occurred while fetching block receipts: {err}")

# # Call function with hash
# block_number = 27085691
# receipts = get_block_receipts(block_number)
# print(receipts)







# from web3 import AsyncWeb3, AsyncHTTPProvider, Web3, HTTPProvider
# import asyncio

# # rpc_url="https://eth.llamarpc.com"
# rpc_url="https://eth-mainnet-public.unifra.io"

# tx_hash = '0x218b632d932371478d1ae5a01620ebab1a2030f9dad6f8fba4a044ea6335a57e'

# w3_async = AsyncWeb3(AsyncHTTPProvider(rpc_url))
# w3_sync = Web3(HTTPProvider(rpc_url))


# async def block_number():
#     bnum = await w3_async.eth.get_block_number()
#     print(bnum)

# def trace_transaction():
#     trace = w3_sync.tracing.trace_raw_transaction(tx_hash)
#     print(trace)

# if __name__ == "__main__":
#     # asyncio.run(block_number())
#     # asyncio.run(trace_transaction())
#     trace_transaction()



# active_env =config.environment
# print(config.chain.name)
# print(config.chain.rpc_urls)

# CHAIN_NAME = config.chain.name
# RPC_URL = config.chain.rpc_urls[0]
# print(CHAIN_NAME)
# print(RPC_URL)

# project_root = Path(__file__).resolve().parent.parent
# print(project_root)
    # async def trace_replay_block_transactions(self, block_number: int) -> None:
    #     block = await self.w3.eth.get_block(block_number, full_transactions=True)
    #     for tx in block['transactions']:
    #         await self.trace_replay_transaction(tx['hash'])

    # async def trace_block(self, block_number: int) -> None:
    #     await self.trace_replay_block_transactions(block_number)

    # async def trace_transaction(self, tx_hash: str) -> None:
    #     await self.trace_transaction(tx_hash)

    # async def trace_call(self, tx_hash: str) -> None:
    #     await self.trace_call(tx_hash)

    # async def trace_raw_transaction(self, tx_hash: str) -> None:
    #     await self.trace_raw_transaction(tx_hash)

# if __name__ == "__main__":
#     evm_indexer = EVMIndexer(rpc_url="https://mainnet.era.zksync.io", chain_type=ChainType.ERA)
#     evm_indexer.trace_block(100000000)










# ############################################################
# Tx receipt testing

# rpc_url = "https://mainnet.era.zksync.io"
# rpc_url = "https://arbitrum.llamarpc.com"
# transaction_hash = '0xcddb8835cfa4c9c4fb2e4c4b50d76f028d4c8c4635b552c6978d957b650df1c0'
# transaction_hash = '0xff3f318604571a76085da9cbcb7039f4f4fdc7e4fe4d6e0b2edbae51d66bb847'
# w3_async = AsyncWeb3(AsyncHTTPProvider(rpc_url))
# w3_sync = Web3(HTTPProvider(rpc_url))
# receipt = w3_sync.eth.get_transaction_receipt(transaction_hash)
# print(receipt)

# block = w3_sync.eth.get_block(277171550, full_transactions=True)
# print(block)


# w3_async = AsyncWeb3(AsyncHTTPProvider(rpc_url))
# w3_sync = Web3(HTTPProvider(rpc_url))

import asyncio
rpc_url = "https://arbitrum.llamarpc.com"
w3_async = AsyncWeb3(AsyncHTTPProvider(rpc_url))

async def get_block_number():
    number = await w3_async.eth.get_block_number()
    return number

async def get_block(number: int):
    block = await w3_async.eth.get_block(number, full_transactions=True)
    return block

async def get_transaction_receipt(hash: str):
    receipt = await w3_async.eth.get_transaction_receipt(hash)
    return receipt

async def main():
    number = await get_block_number()
    block = await get_block(number)
    receipt = await get_transaction_receipt(block['transactions'][0]['hash'])
    print(receipt)

if __name__ == "__main__":
    asyncio.run(main())