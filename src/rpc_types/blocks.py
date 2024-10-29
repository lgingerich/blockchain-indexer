from typing import List, Optional
from eth_typing import (
    BlockNumber,
    HexStr,
)
from web3.types import Wei
from pydantic import BaseModel

class Withdrawal(BaseModel):
    address: str
    amount: int
    index: int
    validator_index: int

class BaseBlock(BaseModel):
    model_config = {
        "arbitrary_types_allowed": False
    }
    
    base_fee_per_gas: Optional[Wei] = None
    difficulty: int
    extra_data: Optional[HexStr] = None
    gas_limit: Wei
    gas_used: Wei
    hash: HexStr
    logs_bloom: HexStr
    miner: str
    mix_hash: HexStr
    nonce: HexStr
    number: BlockNumber
    parent_hash: HexStr
    receipts_root: HexStr
    sha3_uncles: HexStr
    size: int
    state_root: HexStr
    block_time: str
    block_date: str
    total_difficulty: int
    transactions: List[HexStr] = []
    transactions_root: HexStr
    uncles: List[HexStr] = []

class ArbitrumBlock(BaseBlock):
    l1_block_number: int
    send_count: Optional[int] = None
    send_root: Optional[HexStr] = None

class EthereumBlock(BaseBlock):
    blob_gas_used: Optional[int] = None
    excess_blob_gas: Optional[int] = None
    parent_beacon_block_root: Optional[HexStr] = None
    withdrawals: Optional[List[Withdrawal]] = []
    withdrawals_root: Optional[HexStr] = None

class ZKsyncBlock(BaseBlock):
    l1_batch_number: Optional[int] = None
    l1_batch_timestamp: Optional[int] = None
    seal_fields: List[HexStr] = []