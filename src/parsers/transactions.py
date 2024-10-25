from rpc_types import EthereumTransaction, ArbitrumTransaction, ZKsyncTransaction

class BaseTransactionParser:
    @staticmethod
    def parse_raw(raw_tx: dict) -> dict:
        return {
            'blockHash': raw_tx['blockHash'].hex(),
            'blockNumber': raw_tx['blockNumber'],
            'chainId': raw_tx.get('chainId'),
            'from_address': raw_tx['from'],
            'gas': raw_tx['gas'],
            'gasPrice': raw_tx['gasPrice'],
            'hash': raw_tx['hash'].hex(),
            'input': raw_tx['input'].hex(),
            'nonce': raw_tx['nonce'],
            'r': raw_tx.get('r'),
            's': raw_tx.get('s'),
            'to_address': raw_tx['to'],
            'transactionIndex': raw_tx['transactionIndex'],
            'type': raw_tx['type'],
            'v': raw_tx.get('v'),
            'value': raw_tx['value']
        }

class EthereumTransactionParser(BaseTransactionParser):
    @staticmethod
    def parse_raw(raw_tx: dict) -> dict:
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
    def parse_raw(raw_tx: dict) -> dict:
        # Since ArbitrumTransaction has no additional fields,
        # we just return the base parsed transaction
        return BaseTransactionParser.parse_raw(raw_tx)

class ZKsyncTransactionParser(BaseTransactionParser):
    @staticmethod
    def parse_raw(raw_tx: dict) -> dict:
        parsed = BaseTransactionParser.parse_raw(raw_tx)
        parsed.update({
            'l1BatchNumber': raw_tx['l1BatchNumber'],
            'l1BatchTxIndex': raw_tx['l1BatchTxIndex'],
            'maxFeePerGas': raw_tx['maxFeePerGas'],
            'maxPriorityFeePerGas': raw_tx['maxPriorityFeePerGas']
        })
        return parsed

# Mapping to connect types with their parsers
TRANSACTION_PARSERS = {
    ArbitrumTransaction: ArbitrumTransactionParser,
    EthereumTransaction: EthereumTransactionParser,
    ZKsyncTransaction: ZKsyncTransactionParser,
}

