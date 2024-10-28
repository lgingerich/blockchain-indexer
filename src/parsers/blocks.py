from rpc_types import EthereumBlock, ArbitrumBlock, ZKsyncBlock
from utils import hex_to_str

class BaseBlockParser:
    @staticmethod
    def parse_raw(raw_block: dict) -> dict:
        return {
            'baseFeePerGas': raw_block.get('baseFeePerGas'),
            'difficulty': raw_block['difficulty'],
            'extraData': hex_to_str(raw_block['extraData']),
            'gasLimit': raw_block['gasLimit'],
            'gasUsed': raw_block['gasUsed'],
            'hash': hex_to_str(raw_block['hash']),
            'logsBloom': hex_to_str(raw_block['logsBloom']),
            'miner': str(raw_block['miner']),
            'mixHash': hex_to_str(raw_block['mixHash']),
            'nonce': hex_to_str(raw_block['nonce']),
            'number': raw_block['number'],
            'parentHash': hex_to_str(raw_block['parentHash']),
            'receiptsRoot': hex_to_str(raw_block['receiptsRoot']),
            'sha3Uncles': hex_to_str(raw_block['sha3Uncles']),
            'size': raw_block['size'],
            'stateRoot': hex_to_str(raw_block['stateRoot']),
            'timestamp': raw_block['timestamp'],
            'totalDifficulty': raw_block['totalDifficulty'],
            'transactions': [hex_to_str(tx['hash']) for tx in raw_block['transactions']],
            'transactionsRoot': hex_to_str(raw_block['transactionsRoot']),
            'uncles': [hex_to_str(uncle) for uncle in raw_block['uncles']]
        }
    
class ArbitrumBlockParser(BaseBlockParser):
    @staticmethod
    def parse_raw(raw_block: dict) -> ArbitrumBlock:
        parsed = BaseBlockParser.parse_raw(raw_block)
        parsed.update({
            'l1BlockNumber': raw_block['l1BlockNumber'],
            'sendCount': raw_block.get('sendCount'),
            'sendRoot': hex_to_str(raw_block.get('sendRoot')) if raw_block.get('sendRoot') else None,
        })
        return parsed

class EthereumBlockParser(BaseBlockParser):
    @staticmethod
    def parse_raw(raw_block: dict) -> EthereumBlock:
        parsed = BaseBlockParser.parse_raw(raw_block)
        parsed.update({
            'blobGasUsed': raw_block.get('blobGasUsed'),
            'excessBlobGas': raw_block.get('excessBlobGas'),
            'parentBeaconBlockRoot': hex_to_str(raw_block.get('parentBeaconBlockRoot')) if raw_block.get('parentBeaconBlockRoot') else None,
            'withdrawals': raw_block.get('withdrawals'),
            'withdrawalsRoot': hex_to_str(raw_block.get('withdrawalsRoot')) if raw_block.get('withdrawalsRoot') else None,
        })
        return parsed
    
class ZKsyncBlockParser(BaseBlockParser):
    @staticmethod
    def parse_raw(raw_block: dict) -> ZKsyncBlock:
        parsed = BaseBlockParser.parse_raw(raw_block)
        parsed.update({
            'l1BatchNumber': int(raw_block['l1BatchNumber'], 16) if raw_block.get('l1BatchNumber') else None, # convert hex to int
            'l1BatchTimestamp': int(raw_block['l1BatchTimestamp'], 16) if raw_block.get('l1BatchTimestamp') else None, # convert hex to int
            'sealFields': [hex_to_str(sf) for sf in raw_block['sealFields']],
        })
        return parsed
