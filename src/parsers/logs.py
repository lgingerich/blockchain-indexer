from rpc_types import EthereumLog, ArbitrumLog, ZKsyncLog
from utils import hex_to_str, unix_to_utc

class BaseLogParser:
    @staticmethod
    def parse_raw(raw_log: dict, block_timestamp: int) -> dict:
        return {
            'address': str(raw_log['address']),
            'block_hash': hex_to_str(raw_log['blockHash']),
            'block_number': raw_log['blockNumber'],
            'block_time': unix_to_utc(block_timestamp, date_only=False),
            'block_date': unix_to_utc(block_timestamp, date_only=True),
            'data': hex_to_str(raw_log['data']),
            'log_index': raw_log['logIndex'],
            'removed': raw_log['removed'],
            'topics': [hex_to_str(topic) for topic in raw_log['topics']],
            'transaction_hash': hex_to_str(raw_log['transactionHash']),
            'transaction_index': raw_log['transactionIndex']
        }
    
class ArbitrumLogParser(BaseLogParser):
    @staticmethod
    def parse_raw(raw_log: dict, block_timestamp: int) -> ArbitrumLog:
        # Arbitrum logs are identical to base logs
        return BaseLogParser.parse_raw(raw_log, block_timestamp)
    
class EthereumLogParser(BaseLogParser):
    @staticmethod
    def parse_raw(raw_log: dict, block_timestamp: int) -> EthereumLog:
        # Ethereum logs are identical to base logs
        return BaseLogParser.parse_raw(raw_log, block_timestamp)
    
class ZKsyncLogParser(BaseLogParser):
    @staticmethod
    def parse_raw(raw_log: dict, block_timestamp: int) -> ZKsyncLog:
        parsed = BaseLogParser.parse_raw(raw_log, block_timestamp)
        parsed.update({
            'l1_batch_number': int(raw_log['l1BatchNumber'], 16) if raw_log.get('l1BatchNumber') else None, # convert hex to int
            'log_type': raw_log['logType'],
            'transaction_log_index': int(raw_log['transactionLogIndex'], 16) if raw_log.get('transactionLogIndex') else None # convert hex to int
        })
        return parsed
    