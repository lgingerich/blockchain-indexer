from rpc_types import EthereumLog, ArbitrumLog, ZKsyncLog

class BaseLogParser:
    @staticmethod
    def parse_raw(raw_log: dict) -> dict:
        pass
    
class EthereumLogParser(BaseLogParser):
    @staticmethod
    def parse_raw(raw_log: dict) -> dict:
        pass
    
class ArbitrumLogParser(BaseLogParser):
    @staticmethod
    def parse_raw(raw_log: dict) -> dict:
        pass
    
class ZKsyncLogParser(BaseLogParser):
    @staticmethod
    def parse_raw(raw_log: dict) -> dict:
        pass
    
# Mapping to connect types with their parsers
LOG_PARSERS = {
    ArbitrumLog: ArbitrumLogParser,
    EthereumLog: EthereumLogParser,
    ZKsyncLog: ZKsyncLogParser,
}
