from pydantic import BaseModel
from datetime import datetime, date
from typing import List, Optional


class BaseLog(BaseModel):
    model_config = {
        "arbitrary_types_allowed": False,
        # "validate_all": False # Temporarily disable validation
    }
    
    address: str
    block_hash: str
    block_number: int
    block_time: datetime
    block_date: date
    data: str
    log_index: int
    removed: bool
    topics: List[str]
    transaction_hash: str
    transaction_index: int

# Same as BaseTransaction — keep here for clarity and completeness
class ArbitrumLog(BaseLog):
    pass

# Same as BaseTransaction — keep here for clarity and completeness
class EthereumLog(BaseLog):
    pass

class ZKsyncLog(BaseLog):
    l1_batch_number: Optional[int] = None
    log_type: Optional[str] = None
    transaction_log_index: Optional[int] = None