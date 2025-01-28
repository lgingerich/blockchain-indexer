# Blockchain Indexer

## Description

The Blockchain Indexer processes blockchain data from various EVM-compatible chains.
Features include:
- Continuous processing of blockchain data with configurable delay from chain head
- Support for multiple EVM chains including Ethereum, Arbitrum, Optimism, and ZKSync
- Flexible deployment options for local development and production environments
- Customizable chain configurations

> ⚠️ **Note**: This project is in active development and may have frequent breaking changes. It is not recommended for production use yet.


## Prerequisites
- Rust 1.75+ 
- Cargo
- Docker and Docker Compose (for local containerized setup)
- Terraform (for cloud deployment)


## TO DO

### Features
- [ ] Add support for more chains (e.g. Optimism, Arbitrum)
- [ ] Add support for more storage options (e.g. S3, Postgres)
- [ ] Add reorg handling

### Performance Improvements
- [X] Make data storage inserts non-blocking
- [ ] Add batched RPC calls
- [ ] Remove clones

### Misc Improvements
- [ ] `serde_yaml` is no longer maintained but doesn't have a good replacement yet. Check for other possibly unmaintained crates.
- [ ] Add benchmarks
- [ ] Finish migration to enum-based error handling with `thiserror`


## Indexer Configuration

The indexer is configured through a single `config.yml` file. To get started:

1. Copy the example configuration file:
```bash
cp config.yml.example config.yml
```

2. Edit `config.yml` with your settings:
- **Project Name**: Name used for cloud resources and logging
- **Chain Settings**: 
  - `chain_name`: The blockchain to index (e.g., "ethereum", "zksync")
  - `chain_id`: The network ID (e.g., 1 for Ethereum mainnet, 324 for ZKSync)
  - `chain_tip_buffer`: Number of blocks to stay behind the chain head
- **RPC**: URL for your blockchain node (not all RPC providers have been tested)
- **Datasets**: Choose which data types to index:
  - blocks
  - transactions
  - logs
  - traces
- **Metrics**: Enable or disable metrics collection

The actual `config.yml` file is excluded from version control. See `config.yml.example` for a template with all supported options.


## Deployment Options

### 1. Local Rust Setup
Run directly on your machine using Cargo:
```bash
# Clone repository
git clone https://github.com/lgingerich/blockchain-indexer.git
cd blockchain-indexer

# Build and run in development mode
cargo run

# Build and run with optimizations
cargo run --release
```

### 2. Local Docker Setup
Run the indexer using Docker Compose:

```bash
# Clone repository
git clone https://github.com/lgingerich/blockchain-indexer.git
cd blockchain-indexer

# Start the indexer
docker compose up

# Start the indexer with a config file at a custom path
CONFIG_SOURCE=path/to/your/other-config.yml docker compose up
```

### 3. Cloud Deployment with Terraform

#### Authentication Setup
Configure authentication before deploying:

```bash
# Login to Google Cloud
gcloud auth login

# Set up application default credentials
gcloud auth application-default login
```

#### Configuration
1. Copy the example variables file to create your own:
```bash
cp terraform/terraform.tfvars.example terraform/terraform.tfvars
```

2. Edit `terraform/terraform.tfvars` with your specific values:
```terraform
region                  = "us-central1"    # Required: GCP region for deployment
zone                    = "us-central1-a"  # Required: GCP zone within the region
machine_type            = "e2-medium"      # Required: GCP machine type for the VM
create_service_account  = false
```

#### Deploy Infrastructure
Deploy the indexer to Google Cloud Platform:

```bash
# Navigate to terraform directory
cd terraform

# Initialize Terraform
terraform init

# Review the deployment plan
terraform plan

# Deploy the infrastructure
terraform apply
```

## Performance Benchmarking
Note: Always run performance tests with `cargo run --release`

|  Date  | GitHub Commit | Chain | Block Range | RPC | Storage | Total Time (sec) | Blocks/sec | Notes |
|--------|---------------|-------|-------------|-----|---------|------------------|------------|-------|
| 2025-01-21 | [c105b9d2840ec8f3b35e091deb945fbf5551816d](https://github.com/lgingerich/blockchain-indexer/commit/c105b9d2840ec8f3b35e091deb945fbf5551816d) | Ethereum | 10,000,000 - 10,001,000 | DRPC (Free) | BigQuery, 100 Blocks per Insert | 825.37 | 1.21 | |
| 2025-01-21 | [4997e835156c96ff533071301c2eefffe9a35906](https://github.com/lgingerich/blockchain-indexer/commit/4997e835156c96ff533071301c2eefffe9a35906) | Ethereum | 10,000,000 - 10,001,000 | DRPC (Free) | BigQuery, 100 Blocks per Insert | 385.50 | 2.59 | |
| | | | | | |

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
