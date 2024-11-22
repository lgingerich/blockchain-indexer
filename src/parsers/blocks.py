from data_types import EthereumBlock, ArbitrumBlock, ZKsyncBlock
from utils.utils import hex_to_str, unix_to_utc

class BaseBlockParser:
    @staticmethod
    def parse_raw(raw_block: dict) -> dict:
        return {
            'base_fee_per_gas': raw_block.get('baseFeePerGas'),
            'block_hash': hex_to_str(raw_block.get('hash')),
            'block_number': raw_block.get('number'),
            'block_date': unix_to_utc(raw_block.get('timestamp'), date_only=True),
            'block_time': unix_to_utc(raw_block.get('timestamp'), date_only=False),
            'difficulty': raw_block.get('difficulty'),
            'extra_data': hex_to_str(raw_block.get('extraData')) if raw_block.get('extraData') else None,
            'gas_limit': raw_block.get('gasLimit'),
            'gas_used': raw_block.get('gasUsed'),
            'logs_bloom': hex_to_str(raw_block.get('logsBloom')),
            'miner': str(raw_block.get('miner')),
            'mix_hash': hex_to_str(raw_block.get('mixHash')),
            'nonce': hex_to_str(raw_block.get('nonce')),

            'parent_hash': hex_to_str(raw_block.get('parentHash')),
            'receipts_root': hex_to_str(raw_block.get('receiptsRoot')),
            'sha3_uncles': hex_to_str(raw_block.get('sha3Uncles')),
            'size': raw_block.get('size'),
            'state_root': hex_to_str(raw_block.get('stateRoot')),
            'total_difficulty': raw_block.get('totalDifficulty'),
            'transactions': [hex_to_str(tx['hash']) for tx in raw_block.get('transactions', [])],
            'transactions_root': hex_to_str(raw_block.get('transactionsRoot')),
            'uncles': [hex_to_str(uncle) for uncle in raw_block.get('uncles', [])]
        }

class ArbitrumBlockParser(BaseBlockParser):
    @staticmethod
    def parse_raw(raw_block: dict) -> ArbitrumBlock:
        parsed = BaseBlockParser.parse_raw(raw_block)
        parsed.update({
            'l1_block_number': int(raw_block['l1BlockNumber'], 16) if raw_block.get('l1BlockNumber') else None, # convert hex to int
            'send_count': raw_block.get('sendCount'),
            'send_root': hex_to_str(raw_block.get('sendRoot')) if raw_block.get('sendRoot') else None,
        })
        return parsed

class EthereumBlockParser(BaseBlockParser):
    @staticmethod
    def parse_raw(raw_block: dict) -> EthereumBlock:
        parsed = BaseBlockParser.parse_raw(raw_block)
        parsed.update({
            'blob_gas_used': raw_block.get('blobGasUsed'),
            'excess_blob_gas': raw_block.get('excessBlobGas'),
            'parent_beacon_block_root': hex_to_str(raw_block.get('parentBeaconBlockRoot')) if raw_block.get('parentBeaconBlockRoot') else None,
            'withdrawals': raw_block.get('withdrawals'),
            'withdrawals_root': hex_to_str(raw_block.get('withdrawalsRoot')) if raw_block.get('withdrawalsRoot') else None,
        })
        return parsed
    
class ZKsyncBlockParser(BaseBlockParser):
    @staticmethod
    def parse_raw(raw_block: dict) -> ZKsyncBlock:
        parsed = BaseBlockParser.parse_raw(raw_block)
        parsed.update({
            'l1_batch_number': int(raw_block['l1BatchNumber'], 16) if raw_block.get('l1BatchNumber') else None, # convert hex to int
            'l1_batch_time': unix_to_utc(int(raw_block['l1BatchTimestamp'], 16), date_only=False) if raw_block.get('l1BatchTimestamp') else None, # convert hex to int to utc timestamp
            'seal_fields': [hex_to_str(sf) for sf in raw_block['sealFields']],
        })
        return parsed
