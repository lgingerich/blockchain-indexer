import json
import os
from typing import List
from threading import Lock

class MissingBlockTracker:
    """
    A thread-safe tracker for blocks with missing data that need to be retried.
    """
    
    def __init__(self, filepath: str = "missing_blocks.json"):
        self.filepath = filepath
        self.lock = Lock()
        self.missing_blocks = self._load_state()
    
    def _load_state(self) -> List[int]:
        if not os.path.exists(self.filepath):
            return []
        with open(self.filepath, 'r') as f:
            try:
                data = json.load(f)
                return data.get("missing_blocks", [])
            except json.JSONDecodeError:
                return []
    
    def _save_state(self) -> None:
        with open(self.filepath, 'w') as f:
            json.dump({"missing_blocks": self.missing_blocks}, f, indent=4)
    
    def add_block(self, block_number: int) -> None:
        with self.lock:
            if block_number not in self.missing_blocks:
                self.missing_blocks.append(block_number)
                self._save_state()
    
    def remove_block(self, block_number: int) -> None:
        with self.lock:
            if block_number in self.missing_blocks:
                self.missing_blocks.remove(block_number)
                self._save_state()
    
    def get_first_block(self) -> int | None:
        with self.lock:
            if self.missing_blocks:
                return self.missing_blocks[0]
            return None
    
    def get_all_blocks(self) -> List[int]:
        with self.lock:
            return list(self.missing_blocks)