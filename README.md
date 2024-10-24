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

- Because L2 blocks are not immediately finalized on L1, must either:
    - wait for L1 to finalize before indexing
    - reindex ~24 hours later
    - get data without the L1 specific fields

- Need to use transaction receipts also

- Optimism has this error for some earlier blocks:
    ```
    web3.exceptions.ExtraDataLengthError: The field extraData is 97 bytes, but should be 32. It is quite likely that you are connected to a POA chain. Refer to http://web3py.readthedocs.io/en/stable/middleware.html#proof-of-authority for more details. The full extraData is: HexBytes('0xd98301090a846765746889676f312e31352e3133856c696e75780000000000009c3827892825f0825a7e329b6913b84c9e4f89168350aff0939e0e6609629f2e7f07f2aeb62acbf4b16a739cab68866f4880ea406583a4b28a59d4f55dc2314e00')
    ```

    - Could it be related to OVM1?

- In some Ethereum transaction output, some transactions do not have an `access_list` field. Shouldn't this always be
included?

- Some values (specifically in zksync) are returned as hex but I want as integer. Handle this conversion.
