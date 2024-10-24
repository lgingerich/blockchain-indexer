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
