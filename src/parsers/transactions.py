from rpc_types import EthereumTransaction, ArbitrumTransaction, ZKsyncTransaction
from utils import hex_to_str

class BaseTransactionParser:
    @staticmethod
    def parse_raw(raw_tx: dict) -> dict:
        return {
            'block_hash': hex_to_str(raw_tx['blockHash']),
            'block_number': raw_tx['blockNumber'],
            'chain_id': raw_tx.get('chainId'),
            'from_address': str(raw_tx['from']),
            'gas': raw_tx['gas'],
            'gas_price': raw_tx['gasPrice'],
            'hash': hex_to_str(raw_tx['hash']),
            'input': hex_to_str(raw_tx['input']),
            'nonce': raw_tx['nonce'],
            'r': hex_to_str(raw_tx.get('r')) if raw_tx.get('r') else None,
            's': hex_to_str(raw_tx.get('s')) if raw_tx.get('s') else None,
            'to_address': raw_tx['to'],
            'transaction_index': raw_tx['transactionIndex'],
            'type': raw_tx['type'],
            'v': raw_tx.get('v'),
            'value': raw_tx['value']
        }

class EthereumTransactionParser(BaseTransactionParser):
    @staticmethod
    def parse_raw(raw_tx: dict) -> EthereumTransaction:
        parsed = BaseTransactionParser.parse_raw(raw_tx)
        parsed.update({
            'access_list': raw_tx.get('accessList'),
            'blob_versioned_hashes': raw_tx.get('blobVersionedHashes'),
            'max_fee_per_blob_gas': raw_tx.get('maxFeePerBlobGas'),
            'max_fee_per_gas': raw_tx.get('maxFeePerGas'),
            'max_priority_fee_per_gas': raw_tx.get('maxPriorityFeePerGas'),
            'y_parity': raw_tx.get('yParity')
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
            'l1_batch_number': int(raw_tx['l1BatchNumber'], 16) if raw_tx.get('l1BatchNumber') else None, # convert hex to int
            'l1_batch_tx_index': int(raw_tx['l1BatchTxIndex'], 16) if raw_tx.get('l1BatchTxIndex') else None, # convert hex to int
            'max_fee_per_gas': raw_tx['maxFeePerGas'],
            'max_priority_fee_per_gas': raw_tx['maxPriorityFeePerGas']
        })
        return parsed
