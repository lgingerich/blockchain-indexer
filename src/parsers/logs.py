from rpc_types import EthereumLog, ArbitrumLog, ZKsyncLog

class BaseLogParser:
    @staticmethod
    def parse_raw(raw_log: dict) -> dict:
        return {
            'address': raw_log['address'],
            'blockHash': raw_log['blockHash'].hex(),
            'blockNumber': raw_log['blockNumber'],
            'data': raw_log['data'].hex(),
            'logIndex': raw_log['logIndex'],
            'removed': raw_log['removed'],
            'topics': [topic.hex() for topic in raw_log['topics']],
            'transactionHash': raw_log['transactionHash'].hex(),
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
            'blockTimestamp': int(raw_log['blockTimestamp'], 16) # convert hex to int
        })
        return parsed
    
class ZKsyncLogParser(BaseLogParser):
    @staticmethod
    def parse_raw(raw_log: dict) -> ZKsyncLog:
        parsed = BaseLogParser.parse_raw(raw_log)
        parsed.update({
            'blockTimestamp': int(raw_log['blockTimestamp'], 16), # convert hex to int
            'l1BatchNumber': int(raw_log['l1BatchNumber'], 16), # convert hex to int
            'logType': raw_log['logType'],
            'transactionLogIndex': int(raw_log['transactionLogIndex'], 16) # convert hex to int
        })
        return parsed
    
# Mapping to connect types with their parsers
LOG_PARSERS = {
    ArbitrumLog: ArbitrumLogParser,
    EthereumLog: EthereumLogParser,
    ZKsyncLog: ZKsyncLogParser,
}
