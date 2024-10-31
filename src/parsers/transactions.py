from rpc_types import EthereumTransaction, ArbitrumTransaction, ZKsyncTransaction
from utils import hex_to_str, unix_to_utc

class BaseTransactionParser:
    @staticmethod
    def parse_raw(raw_tx: dict, block_timestamp: int, receipt: dict) -> dict:
        """Parse transaction data from both transaction and receipt"""
        return {
            # Fields from transaction
            'block_hash': hex_to_str(raw_tx['blockHash']),
            'block_number': raw_tx['blockNumber'],
            'block_time': unix_to_utc(block_timestamp, date_only=False),
            'block_date': unix_to_utc(block_timestamp, date_only=True),
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
            'value': str(raw_tx['value']),

            # Fields from receipt
            'status': receipt['status'],
            'cumulative_gas_used': receipt['cumulativeGasUsed'],
            'effective_gas_price': receipt['effectiveGasPrice'],
            'gas_used': receipt['gasUsed'],
            'logs_bloom': hex_to_str(receipt['logsBloom']),
            'contract_address': receipt.get('contractAddress'),
        }

class ArbitrumTransactionParser(BaseTransactionParser):
    @staticmethod
    def parse_raw(raw_tx: dict, block_timestamp: int, receipt: dict) -> ArbitrumTransaction:
        parsed = BaseTransactionParser.parse_raw(raw_tx, block_timestamp, receipt)
        parsed.update({
            # Additional receipt fields specific to Arbitrum
            'blob_gas_used': receipt.get('blobGasUsed'),
            'l1_block_number': receipt.get('l1BlockNumber'),
            'gas_used_for_l1': receipt.get('gasUsedForL1'),
        })
        return parsed

class EthereumTransactionParser(BaseTransactionParser):
    @staticmethod
    def parse_raw(raw_tx: dict, block_timestamp: int, receipt: dict) -> EthereumTransaction:
        parsed = BaseTransactionParser.parse_raw(raw_tx, block_timestamp, receipt)
        parsed.update({
            'access_list': raw_tx.get('accessList', []),
            'blob_versioned_hashes': raw_tx.get('blobVersionedHashes', []),
            'max_fee_per_blob_gas': raw_tx.get('maxFeePerBlobGas'),
            'max_fee_per_gas': raw_tx.get('maxFeePerGas'),
            'max_priority_fee_per_gas': raw_tx.get('maxPriorityFeePerGas'),
            'y_parity': raw_tx.get('yParity')
        })
        return parsed

class ZKsyncTransactionParser(BaseTransactionParser):
    @staticmethod
    def parse_raw(raw_tx: dict, block_timestamp: int, receipt: dict) -> ZKsyncTransaction:
        parsed = BaseTransactionParser.parse_raw(raw_tx, block_timestamp, receipt)
        parsed.update({
            # Fields from transaction
            'l1_batch_number': int(raw_tx['l1BatchNumber'], 16) if raw_tx.get('l1BatchNumber') else None,
            'l1_batch_tx_index': int(raw_tx['l1BatchTxIndex'], 16) if raw_tx.get('l1BatchTxIndex') else None,
            'max_fee_per_gas': raw_tx['maxFeePerGas'],
            'max_priority_fee_per_gas': raw_tx['maxPriorityFeePerGas'],
            
            # Fields from receipt
            'root': receipt.get('root', '')
        })
        return parsed
