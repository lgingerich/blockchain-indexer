import datetime

async def process_blocks(block, chain):
    if chain == "ethereum":
        # Ethereum-specific block structure
        # return {
        #     'base_fee_per_gas': block.get('baseFeePerGas', None),
        #     'difficulty': block.get('difficulty', None),
        #     'extra_data': block.get('extraData', None).hex() if block.get('extraData') is not None else None,
        #     'gas_limit': block.get('gasLimit', None),
        #     'gas_used': block.get('gasUsed'),
        #     'block_hash': block.get('hash', None).hex() if block.get('hash') is not None else None,
        #     'logs_bloom': block.get('logsBloom', None).hex() if block.get('logsBloom') is not None else None,
        #     'miner': block.get('miner', None),
        #     'mix_hash': block.get('mixHash', None).hex() if block.get('mixHash') is not None else None,
        #     'nonce': block.get('nonce').hex() if block.get('nonce') is not None else None,
        #     'number': block.get('number'),
        #     'parent_hash': block.get('parentHash').hex() if block.get('parentHash') is not None else None,
        #     'receipts_root': block.get('receiptsRoot').hex() if block.get('receiptsRoot') is not None else None,
        #     'transactions_root': block.get('transactionsRoot').hex() if block.get('transactionsRoot') is not None else None,
        #     'sha3_uncles': block.get('sha3Uncles').hex() if block.get('sha3Uncles') is not None else None,
        #     'size': block.get('size'),
        #     'state_root': block.get('stateRoot').hex() if block.get('stateRoot') is not None else None,
        #     'timestamp': block.get('timestamp'),
        #     'uncles': block.get('uncles'),
        #     # 'total_difficulty': Decimal(block.totalDifficulty),
        #     # 'block_time': datetime.datetime.utcfromtimestamp(block.get('timestamp')).strftime('%Y-%m-%d %H:%M:%S') if block.get('timestamp') is not None else None,
        #     # 'block_date': datetime.datetime.utcfromtimestamp(block.get('timestamp')).strftime('%Y-%m-%d') if block.get('timestamp') is not None else None
        # }
        return {
            'base_fee_per_gas': int(block.get('baseFeePerGas', 0)) if block.get('baseFeePerGas') is not None else None,
            'difficulty': int(block.get('difficulty', 0)) if block.get('difficulty') is not None else None,
            'extra_data': block.get('extraData', None).hex() if block.get('extraData') is not None else None,
            'gas_limit': int(block.get('gasLimit', 0)) if block.get('gasLimit') is not None else None,
            'gas_used': int(block.get('gasUsed', 0)) if block.get('gasUsed') is not None else None,
            'block_hash': block.get('hash', None).hex() if block.get('hash') is not None else None,
            'logs_bloom': block.get('logsBloom', None).hex() if block.get('logsBloom') is not None else None,
            'miner': block.get('miner', None),
            'mix_hash': block.get('mixHash', None).hex() if block.get('mixHash') is not None else None,
            'nonce': block.get('nonce', None).hex() if block.get('nonce') is not None else None,
            'number': int(block.get('number', 0)) if block.get('number') is not None else None,
            'parent_hash': block.get('parentHash', None).hex() if block.get('parentHash') is not None else None,
            'receipts_root': block.get('receiptsRoot', None).hex() if block.get('receiptsRoot') is not None else None,
            'transactions_root': block.get('transactionsRoot', None).hex() if block.get('transactionsRoot') is not None else None,
            'sha3_uncles': block.get('sha3Uncles', None).hex() if block.get('sha3Uncles') is not None else None,
            'size': int(block.get('size', 0)) if block.get('size') is not None else None,
            'state_root': block.get('stateRoot', None).hex() if block.get('stateRoot') is not None else None,
            'timestamp': int(block.get('timestamp', 0)) if block.get('timestamp') is not None else None,
            'uncles': block.get('uncles', [])
        }
    elif chain == "zksync_era":
        return {
            'base_fee_per_gas': block.get('baseFeePerGas', None),
            'difficulty': block.get('difficulty', None),
            'extra_data': block.get('extraData', None).hex() if block.get('extraData') is not None else None,
            'gas_limit': block.get('gasLimit', None),
            'gas_used': block.get('gasUsed'),
            'block_hash': block.get('hash', None).hex() if block.get('hash') is not None else None,
            'logs_bloom': block.get('logsBloom', None).hex() if block.get('logsBloom') is not None else None,
            'miner': block.get('miner', None),
            'mix_hash': block.get('mixHash', None).hex() if block.get('mixHash') is not None else None,
            'nonce': block.get('nonce').hex() if block.get('nonce') is not None else None,
            'number': block.get('number'),
            'l1_batch_number': int(block.get('l1BatchNumber'), 16),
            'parent_hash': block.get('parentHash').hex() if block.get('parentHash') is not None else None,
            'receipts_root': block.get('receiptsRoot').hex() if block.get('receiptsRoot') is not None else None,
            'transactions_root': block.get('transactionsRoot').hex() if block.get('transactionsRoot') is not None else None,
            'sha3_uncles': block.get('sha3Uncles').hex() if block.get('sha3Uncles') is not None else None,
            'size': block.get('size'),
            'state_root': block.get('stateRoot').hex() if block.get('stateRoot') is not None else None,
            'timestamp': block.get('timestamp'),
            'l1_batch_imestamp': int(block.get('l1BatchTimestamp'), 16),
            'uncles': block.get('uncles'),
            'seal_fields': block.get('sealFields'),
            # 'total_difficulty': Decimal(block.totalDifficulty),
            # 'block_time': datetime.datetime.utcfromtimestamp(block.get('timestamp')).strftime('%Y-%m-%d %H:%M:%S') if block.get('timestamp') is not None else None,
            # 'block_date': datetime.datetime.utcfromtimestamp(block.get('timestamp')).strftime('%Y-%m-%d') if block.get('timestamp') is not None else None
        }
    else:
        raise ValueError("Unsupported chain for block structure")


async def process_transactions(tx, chain):
    if chain == "ethereum":
        return {
            'block_hash': tx.get('blockHash', None).hex() if tx.get('blockHash') is not None else None,
            'block_number': tx.get('blockNumber', None),
            'from_address': tx.get('from', None),
            'to_address': tx.get('to', None),
            'gas_limit': tx.get('gas', None),
            'gas_price': tx.get('gasPrice', None),
            'max_fee_per_gas': tx.get('maxFeePerGas', None),
            'max_priority_fee_per_gas': tx.get('maxPriorityFeePerGas', None),
            'transaction_hash': tx.get('hash', None).hex() if tx.get('hash') is not None else None,
            'input': tx.get('input', None).hex() if tx.get('input') is not None else None,
            'nonce': tx.get('nonce', None),
            'transaction_index': tx.get('transactionIndex', None),
            'value': tx.get('value', None),
            'type': tx.get('type', None),
            'access_list': tx.get('accessList', None),
            'chain_id': tx.get('chainId', None),
            'v': tx.get('v', None),
            'r': tx.get('r', None).hex() if tx.get('r') is not None else None,
            's': tx.get('s', None).hex() if tx.get('s') is not None else None,
            'y_parity': tx.get('yParity', None),
        }
    elif chain == "zksync_era":
        return {
            'block_hash': tx.get('blockHash', None).hex() if tx.get('blockHash') is not None else None,
            'block_number': tx.get('blockNumber', None),
            'from_address': tx.get('from', None),
            'to_address': tx.get('to', None),
            'gas_limit': tx.get('gas', None),
            'gas_price': tx.get('gasPrice', None),
            'max_fee_per_gas': tx.get('maxFeePerGas', None),
            'max_priority_fee_per_gas': tx.get('maxPriorityFeePerGas', None),
            'transaction_hash': tx.get('hash', None).hex() if tx.get('hash') is not None else None,
            'input': tx.get('input', None).hex() if tx.get('input') is not None else None,
            'nonce': tx.get('nonce', None),
            'transaction_index': tx.get('transactionIndex', None),
            'value': tx.get('value', None),
            'type': tx.get('type', None),
            'access_list': tx.get('accessList', None),
            'chain_id': tx.get('chainId', None),
            'l1_batch_number': int(tx.get('l1BatchNumber', None), 16),
            'l1_batch_tx_index': int(tx.get('l1BatchTxIndex', None), 16),
            'v': tx.get('v', None),
            'r': tx.get('r', None).hex() if tx.get('r') is not None else None,
            's': tx.get('s', None).hex() if tx.get('s') is not None else None,
            'y_parity': tx.get('yParity', None),
        }
    else:
        raise ValueError("Unsupported chain for transaction structure")


# async def process_transaction_receipts(chain):
#     if chain == "ethereum":
#         return {...}  # Ethereum-specific transaction receipt structure
#     elif chain == "arbitrum":
#         return {...}  # Arbitrum-specific transaction receipt structure
#     else:
#         raise ValueError("Unsupported chain for transaction receipt structure")


async def process_logs(log, topics, chain):
    if chain == "ethereum":
        return {
            'contract_address': log.get('address'),
            'block_hash': log.get('blockHash', None).hex() if log.get('blockHash') is not None else None,
            'block_number': log.get('blockNumber', None),
            'data': log.get('data').hex(),
            'log_index': log.get('logIndex', None),
            'removed': log.get('removed', None),
            'topic0': topics[0] if len(topics) > 0 else None,
            'topic1': topics[1] if len(topics) > 1 else None,
            'topic2': topics[2] if len(topics) > 2 else None,
            'topic3': topics[3] if len(topics) > 3 else None,
            'transaction_index': log.get('transactionIndex', None),
            'transaction_hash': log.get('transactionHash', None).hex() if log.get('transactionHash') is not None else None,
        }
    elif chain == "zksync_era":
        return {
            'contract_address': log.get('address'),
            'block_hash': log.get('blockHash', None).hex() if log.get('blockHash') is not None else None,
            'block_number': log.get('blockNumber', None),
            'l1_batch_number': log.get('l1BatchNumber', None),
            'data': log.get('data').hex(),
            'log_index': log.get('logIndex', None),
            'removed': log.get('removed', None),
            'topic0': topics[0] if len(topics) > 0 else None,
            'topic1': topics[1] if len(topics) > 1 else None,
            'topic2': topics[2] if len(topics) > 2 else None,
            'topic3': topics[3] if len(topics) > 3 else None,
            'transaction_index': log.get('transactionIndex', None),
            'transaction_log_index': int(log.get('transactionLogIndex', None), 16),
            'log_type': log.get('logType', None),
            'transaction_hash': log.get('transactionHash', None).hex() if log.get('transactionHash') is not None else None,
        }
    else:
        raise ValueError("Unsupported chain for log structure")
