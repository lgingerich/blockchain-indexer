# Blockchain Indexer

## Description

The Blockchain Indexer is a tool designed to monitor and process Ethereum blockchain data in real-time. It supports multiple Ethereum-compatible chains, including Ethereum, Arbitrum, Optimism, and ZKSync. The indexer connects to these blockchains using Web3 and retrieves block data asynchronously.

## Features

- Real-time monitoring of Ethereum-compatible blockchains.
- Asynchronous operations using `asyncio` for efficient data retrieval.
- Support for multiple chains: Ethereum, Arbitrum, Optimism, and ZKSync.
- Customizable chain configurations.

## Installation

Clone the repository:

```bash
git clone https://github.com/lgingerich/blockchain-indexer.git
cd blockchain-indexer
```

Set up a virtual environment and install dependencies using Astral UV:

```bash
uv venv
uv install
```

## Usage

Run the indexer:

```bash
python src/main.py
```

This will start the indexer, which will connect to the specified blockchain and begin retrieving block data.

## Configuration

- **Chain Selection**: Modify the `CHAIN_NAME` and `rpc_url` in `src/main.py` to select the desired blockchain and RPC endpoint.
- **Block Retrieval**: The indexer retrieves the current block number and block details using the `EVMIndexer` class.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.






## Notes / To Do

TO DO:
- Add partioning on block date
- Add transaction receipts
- Add automatic Bigquery schema generation for all chains
- Add upsert on zksync (and other chains) l1 block data that is missing from the indexer
    - Track the missing data (by l2 block number)
    - Run as separate "path" beside main indexer and check every 60 seconds if the data is available
        - If the data is available, upsert it to Bigquery
        - Run immediately for all blocks until I hit another missing one, then restart 60 second sleep period
- Make data_manager more modular
- Add config setup file
- Add monitoring metrics
- Fetch logs over a much larger block range
    - Need to test what the optimal size is
- Can web3.py fetch multiple blocks at the same time for get_blocks?
    - If yes, can this also be done in Rust?
- Add traces
