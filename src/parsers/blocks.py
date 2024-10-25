from rpc_types import EthereumBlock, ArbitrumBlock, ZKsyncBlock

class BaseBlockParser:
    @staticmethod
    def parse_raw(raw_block: dict) -> dict:
        return {
            'baseFeePerGas': raw_block.get('baseFeePerGas'),
            'difficulty': raw_block['difficulty'],
            'extraData': raw_block['extraData'].hex(),
            'gasLimit': raw_block['gasLimit'],
            'gasUsed': raw_block['gasUsed'],
            'hash': raw_block['hash'].hex(),
            'logsBloom': raw_block['logsBloom'].hex(),
            'miner': raw_block['miner'],
            'mixHash': raw_block['mixHash'].hex(),
            'nonce': raw_block['nonce'].hex(),
            'number': raw_block['number'],
            'parentHash': raw_block['parentHash'].hex(),
            'receiptsRoot': raw_block['receiptsRoot'].hex(),
            'sha3Uncles': raw_block['sha3Uncles'].hex(),
            'size': raw_block['size'],
            'stateRoot': raw_block['stateRoot'].hex(),
            'timestamp': raw_block['timestamp'],
            'totalDifficulty': raw_block['totalDifficulty'],
            'transactions': [tx.hash for tx in raw_block['transactions']],
            'transactionsRoot': raw_block['transactionsRoot'].hex(),
            'uncles': [uncle.hex() for uncle in raw_block['uncles']]
        }
    
class EthereumBlockParser(BaseBlockParser):
    @staticmethod
    def parse_raw(raw_block: dict) -> dict:
        parsed = BaseBlockParser.parse_raw(raw_block)
        parsed.update({
            'blobGasUsed': raw_block.get('blobGasUsed'),
            'excessBlobGas': raw_block.get('excessBlobGas'),
            'parentBeaconBlockRoot': raw_block.get('parentBeaconBlockRoot'),
            'withdrawals': raw_block.get('withdrawals'),
            'withdrawalsRoot': raw_block.get('withdrawalsRoot'),
        })
        return parsed
    
class ArbitrumBlockParser(BaseBlockParser):
    @staticmethod
    def parse_raw(raw_block: dict) -> dict:
        parsed = BaseBlockParser.parse_raw(raw_block)
        parsed.update({
            'l1BlockNumber': raw_block['l1BlockNumber'],
            'sendCount': raw_block.get('sendCount'),
            'sendRoot': raw_block.get('sendRoot'),
        })
        return parsed
    
class ZKsyncBlockParser(BaseBlockParser):
    @staticmethod
    def parse_raw(raw_block: dict) -> dict:
        parsed = BaseBlockParser.parse_raw(raw_block)
        parsed.update({
            'l1BatchNumber': raw_block.get('l1BatchNumber'),
            'l1BatchTimestamp': raw_block.get('l1BatchTimestamp'),
            'sealFields': raw_block['sealFields'],
        })
        return parsed
    
# Mapping to connect types with their parsers
BLOCK_PARSERS = {
    ArbitrumBlock: ArbitrumBlockParser,
    EthereumBlock: EthereumBlockParser,
    ZKsyncBlock: ZKsyncBlockParser,
}
