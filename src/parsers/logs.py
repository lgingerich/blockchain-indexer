from rpc_types import EthereumLog, ArbitrumLog, ZKsyncLog
from utils import hex_to_str

class BaseLogParser:
    @staticmethod
    def parse_raw(raw_log: dict) -> dict:
        return {
            'address': str(raw_log['address']),
            'blockHash': hex_to_str(raw_log['blockHash']),
            'blockNumber': raw_log['blockNumber'],
            'data': hex_to_str(raw_log['data']),
            'logIndex': raw_log['logIndex'],
            'removed': raw_log['removed'],
            'topics': [hex_to_str(topic) for topic in raw_log['topics']],
            'transactionHash': hex_to_str(raw_log['transactionHash']),
            'transactionIndex': raw_log['transactionIndex']
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
            'blockTimestamp': int(raw_log['blockTimestamp'], 16) if raw_log.get('blockTimestamp') else None, # convert hex to int
        })
        return parsed
    
class ZKsyncLogParser(BaseLogParser):
    @staticmethod
    def parse_raw(raw_log: dict) -> ZKsyncLog:
        parsed = BaseLogParser.parse_raw(raw_log)
        parsed.update({
            'blockTimestamp': int(raw_log['blockTimestamp'], 16) if raw_log.get('blockTimestamp') else None, # convert hex to int
            'l1BatchNumber': int(raw_log['l1BatchNumber'], 16) if raw_log.get('l1BatchNumber') else None, # convert hex to int
            'logType': raw_log['logType'],
            'transactionLogIndex': int(raw_log['transactionLogIndex'], 16) if raw_log.get('transactionLogIndex') else None # convert hex to int
        })
        return parsed
    