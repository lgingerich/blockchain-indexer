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
- Python 3.12+
- uv (for local Python setup)
- Docker and Docker Compose (for local containerized setup)
- Terraform (for cloud deployment)


## TO DO

### Features
- [ ] Add support for more chains (e.g. Optimism, Arbitrum)
- [ ] Add support for more storage options (e.g. S3, Postgres)
- [ ] Add reorg handling

### Performance Improvements
- [ ] Make data storage inserts non-blocking
- [ ] Add batched RPC calls
- [ ] Remove clones


## Indexer Configuration

The indexer is configured through a single `config.yml` file. To get started:

1. Copy the example configuration file:
```bash
cp config.yml.example config.yml
```

2. Edit `config.yml` with your settings:
- **Chain**: Specify a single chain to index (ethereum, arbitrum, or zksync) and its RPC URLs
- **Datasets**: Choose which data types to index (blocks, transactions, and/or logs)
- **Storage**: Configure your data storage settings (Note: Currently GCP Bigquery is the only supported storage option)

The actual `config.yml` file is excluded from version control. See `config.yml.example` for a template with all supported options.


## Deployment Options

### 1. Local Python Setup
Run directly on your machine using uv:
```bash
# Clone repository
git clone https://github.com/lgingerich/blockchain-indexer.git
cd blockchain-indexer

# Setup Python environment
uv venv
uv pip install -e .

# Run indexer
uv run src/main.py
```

### 2. Local Docker Setup
Run the indexer using Docker Compose:

```bash
# Clone repository
git clone https://github.com/lgingerich/blockchain-indexer.git
cd blockchain-indexer

# Start the indexer
docker compose up
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

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
