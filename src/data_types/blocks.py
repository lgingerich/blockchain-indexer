from datetime import datetime, date
from decimal import Decimal
from pydantic import BaseModel
from typing import List, Optional


class Withdrawal(BaseModel):
    model_config = {
        "arbitrary_types_allowed": False,
        # "validate_all": False # Temporarily disable validation
    }

    address: str
    amount: int
    index: int
    validator_index: int

class BaseBlock(BaseModel):
    model_config = {
        "arbitrary_types_allowed": False,
        # "validate_all": False # Temporarily disable validation
    }
    
    base_fee_per_gas: Optional[int] = None
    block_hash: str
    block_number: int
    block_date: date
    block_time: datetime
    difficulty: Decimal # Must be Decimal for BigQuery NUMERIC type
    extra_data: Optional[str] = None
    gas_limit: int
    gas_used: int
    logs_bloom: str
    miner: str
    mix_hash: str
    nonce: str
    parent_hash: str
    receipts_root: str
    sha3_uncles: str
    size: int
    state_root: str
    total_difficulty: Decimal # Must be Decimal for BigQuery NUMERIC type
    transactions: List[str] = []
    transactions_root: str
    uncles: List[str] = []

class ArbitrumBlock(BaseBlock):
    l1_block_number: int
    send_count: Optional[int] = None
    send_root: Optional[str] = None

class EthereumBlock(BaseBlock):
    blob_gas_used: Optional[int] = None
    excess_blob_gas: Optional[int] = None
    parent_beacon_block_root: Optional[str] = None
    withdrawals: Optional[List[Withdrawal]] = []
    withdrawals_root: Optional[str] = None

class ZKsyncBlock(BaseBlock):
    l1_batch_number: Optional[int] = None
    l1_batch_time: Optional[datetime] = None
    seal_fields: List[str] = []