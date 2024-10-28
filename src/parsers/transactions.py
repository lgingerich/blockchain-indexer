from rpc_types import EthereumTransaction, ArbitrumTransaction, ZKsyncTransaction
from utils import hex_to_str

class BaseTransactionParser:
    @staticmethod
    def parse_raw(raw_tx: dict) -> dict:
        return {
            'blockHash': hex_to_str(raw_tx['blockHash']),
            'blockNumber': raw_tx['blockNumber'],
            'chainId': raw_tx.get('chainId'),
            'from_address': str(raw_tx['from']),
            'gas': raw_tx['gas'],
            'gasPrice': raw_tx['gasPrice'],
            'hash': hex_to_str(raw_tx['hash']),
            'input': hex_to_str(raw_tx['input']),
            'nonce': raw_tx['nonce'],
            'r': hex_to_str(raw_tx.get('r')),
            's': hex_to_str(raw_tx.get('s')),
            'to_address': raw_tx['to'],
            'transactionIndex': raw_tx['transactionIndex'],
            'type': raw_tx['type'],
            'v': raw_tx.get('v'),
            'value': raw_tx['value']
        }

class EthereumTransactionParser(BaseTransactionParser):
    @staticmethod
    def parse_raw(raw_tx: dict) -> EthereumTransaction:
        parsed = BaseTransactionParser.parse_raw(raw_tx)
        parsed.update({
            'accessList': raw_tx.get('accessList'),
            'blobVersionedHashes': raw_tx.get('blobVersionedHashes'),
            'maxFeePerBlobGas': raw_tx.get('maxFeePerBlobGas'),
            'maxFeePerGas': raw_tx.get('maxFeePerGas'),
            'maxPriorityFeePerGas': raw_tx.get('maxPriorityFeePerGas'),
            'yParity': raw_tx.get('yParity')
        })
        return parsed

class ArbitrumTransactionParser(BaseTransactionParser):
    @staticmethod
    def parse_raw(raw_tx: dict) -> ArbitrumTransaction:
        # Since ArbitrumTransaction has no additional fields,
        # we just return the base parsed transaction
        return BaseTransactionParser.parse_raw(raw_tx)

class ZKsyncTransactionParser(BaseTransactionParser):
    @staticmethod
    def parse_raw(raw_tx: dict) -> ZKsyncTransaction:
        parsed = BaseTransactionParser.parse_raw(raw_tx)
        parsed.update({
            'l1BatchNumber': int(raw_tx['l1BatchNumber'], 16) if raw_tx.get('l1BatchNumber') else None, # convert hex to int
            'l1BatchTxIndex': int(raw_tx['l1BatchTxIndex'], 16) if raw_tx.get('l1BatchTxIndex') else None, # convert hex to int
            'maxFeePerGas': raw_tx['maxFeePerGas'],
            'maxPriorityFeePerGas': raw_tx['maxPriorityFeePerGas']
        })
        return parsed
