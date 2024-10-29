from rpc_types import EthereumLog, ArbitrumLog, ZKsyncLog
from utils import hex_to_str

class BaseLogParser:
    @staticmethod
    def parse_raw(raw_log: dict) -> dict:
        return {
            'address': str(raw_log['address']),
            'block_hash': hex_to_str(raw_log['blockHash']),
            'block_number': raw_log['blockNumber'],
            'data': hex_to_str(raw_log['data']),
            'log_index': raw_log['logIndex'],
            'removed': raw_log['removed'],
            'topics': [hex_to_str(topic) for topic in raw_log['topics']],
            'transaction_hash': hex_to_str(raw_log['transactionHash']),
            'transaction_index': raw_log['transactionIndex']
        }
    
class ArbitrumLogParser(BaseLogParser):
    @staticmethod
    def parse_raw(raw_log: dict) -> ArbitrumLog:
        # Arbitrum logs are identical to base logs
        return BaseLogParser.parse_raw(raw_log)
    
class EthereumLogParser(BaseLogParser):
    @staticmethod
    def parse_raw(raw_log: dict) -> EthereumLog:
        parsed = BaseLogParser.parse_raw(raw_log)
        parsed.update({
            'block_timestamp': int(raw_log['blockTimestamp'], 16) if raw_log.get('blockTimestamp') else None, # convert hex to int
        })
        return parsed
    
class ZKsyncLogParser(BaseLogParser):
    @staticmethod
    def parse_raw(raw_log: dict) -> ZKsyncLog:
        parsed = BaseLogParser.parse_raw(raw_log)
        parsed.update({
            'block_timestamp': int(raw_log['blockTimestamp'], 16) if raw_log.get('blockTimestamp') else None, # convert hex to int
            'l1_batch_number': int(raw_log['l1BatchNumber'], 16) if raw_log.get('l1BatchNumber') else None, # convert hex to int
            'log_type': raw_log['logType'],
            'transaction_log_index': int(raw_log['transactionLogIndex'], 16) if raw_log.get('transactionLogIndex') else None # convert hex to int
        })
        return parsed
    