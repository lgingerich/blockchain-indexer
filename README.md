# Blockchain Indexer

## Description
The Blockchain Indexer is a tool designed to monitor and process Ethereum blockchain data in real-time. It consists of two main microservices: `Block Checker` and `Block Processor`. The Block Checker listens for new blocks on the Ethereum blockchain using
a WebSocket connection and sends this information to a RabbitMQ queue. The Block Processor then consumes these messages from the queue and processes them accordingly.

## Features
- Real-time monitoring of Ethereum blockchain.
- Efficient queuing of blockchain data using RabbitMQ.
- Separate microservices for checking and processing blocks.
- Dockerized environment for easy deployment and scaling.

## Installation

Clone the repository:

```bash
git clone https://github.com/lgingerich/blockchain-indexer.git
cd blockchain-indexer
```

Build the Docker images:

```bash
docker compose build
```

## Usage

Start the services using Docker Compose:

```bash
docker compose up
```

This will start both the Block Checker and Block Processor services, as well as the RabbitMQ server.

## Configuration

- **RabbitMQ**: The RabbitMQ server is configured in `docker-compose.yml`. Modify the environment variables as needed.
- **Block Checker**: Configured to connect to Ethereum's WebSocket and RabbitMQ. Modify `block_checker/block_checker.py` for specific configurations.
- **Block Processor**: Set up to listen to the RabbitMQ queue. Adjust settings in `block_processor/block_processor.py`.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
