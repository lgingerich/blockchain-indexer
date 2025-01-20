terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 4.0"
    }
  }
}

# Use a local to simplify the email reference
locals {
  service_account_email = var.create_service_account ? (
    google_service_account.vm_service_account[0].email
  ) : data.google_service_account.existing_vm_sa[0].email
}

# Get variables from config file
locals {
 config = yamldecode(file("${path.module}/../config.yml"))
 project_id = local.config.storage.project_id
 chain_name = local.config.chain.name
 metrics_public = try(local.config.monitoring.exposure.type, "local") == "public"
}

provider "google" {
  project = local.project_id
  region  = var.region
  zone    = var.zone
}

# Try to get existing service account
data "google_service_account" "existing_vm_sa" {
  count      = var.create_service_account ? 0 : 1
  account_id = "indexer-vm-sa"
  project    = local.project_id
}

# Create service account if it doesn't exist
resource "google_service_account" "vm_service_account" {
  count        = var.create_service_account ? 1 : 0
  account_id   = "indexer-vm-sa"
  display_name = "Indexer VM Service Account"
  project      = local.project_id
}

# Grant BigQuery permissions to the service account
resource "google_project_iam_member" "bigquery_access" {
  project = local.project_id
  role    = "roles/bigquery.dataEditor"
  member  = "serviceAccount:${local.service_account_email}"
}

# Grant BigQuery Job User permissions to the service account
resource "google_project_iam_member" "bigquery_job_user" {
  project = local.project_id
  role    = "roles/bigquery.jobUser"
  member  = "serviceAccount:${local.service_account_email}"
}

# Grant Cloud Logging permissions to the service account
resource "google_project_iam_member" "logging_access" {
  project = local.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${local.service_account_email}"
}

# Grant Monitoring Viewer permissions to the service account
resource "google_project_iam_member" "monitoring_access" {
  project = local.project_id
  role    = "roles/monitoring.metricWriter"
  member  = "serviceAccount:${local.service_account_email}"
}

# Static IP for Prometheus metrics endpoint
resource "google_compute_address" "metrics_ip" {
  # Only create if metrics should be public
  count        = local.metrics_public ? 1 : 0
  name         = "indexer-${replace(local.chain_name, "_", "-")}-metrics-ip"
  region       = var.region
  description  = "Static IP for Prometheus metrics endpoint"
}

# VPC network
resource "google_compute_network" "vpc_network" {
  name                    = "indexer-${replace(local.chain_name, "_", "-")}-network"
  auto_create_subnetworks = false
}

# Subnet
resource "google_compute_subnetwork" "subnet" {
  name          = "indexer-${replace(local.chain_name, "_", "-")}-subnet"
  ip_cidr_range = "10.0.0.0/24"
  network       = google_compute_network.vpc_network.id
  region        = var.region
}

# IAP SSH firewall rule
resource "google_compute_firewall" "iap_ssh" {
  name    = "allow-iap-ssh-${replace(local.chain_name, "_", "-")}"
  network = google_compute_network.vpc_network.name
  
  allow {
    protocol = "tcp"
    ports    = ["22"]
  }

  source_ranges = ["35.235.240.0/20"] # IAP's IP range
  target_tags   = ["ssh"]
}

# Add Cloud NAT router
resource "google_compute_router" "router" {
  name    = "indexer-${replace(local.chain_name, "_", "-")}-router"
  region  = var.region
  network = google_compute_network.vpc_network.id
}

# Add Cloud NAT config
resource "google_compute_router_nat" "nat" {
  name                               = "indexer-${replace(local.chain_name, "_", "-")}-nat"
  router                             = google_compute_router.router.name
  region                             = var.region
  nat_ip_allocate_option             = "AUTO_ONLY"
  source_subnetwork_ip_ranges_to_nat = "ALL_SUBNETWORKS_ALL_IP_RANGES"
}

# Create the secret
resource "google_secret_manager_secret" "indexer_config" {
  secret_id = "indexer-${replace(local.chain_name, "_", "-")}-config"
  
  replication {
    auto {}
  }
}

# Upload the local config to the secret
resource "google_secret_manager_secret_version" "indexer_config" {
  secret = google_secret_manager_secret.indexer_config.id
  secret_data = file("${path.module}/../config.yml")
}

# Grant the VM's service account access to read the secret
resource "google_secret_manager_secret_iam_member" "secret_access" {
  secret_id = google_secret_manager_secret.indexer_config.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${local.service_account_email}"
}

# VM Instance
resource "google_compute_instance" "indexer_vm" {
  name         = "indexer-${replace(local.chain_name, "_", "-")}"
  machine_type = var.machine_type
  zone         = var.zone
  
  tags = ["ssh"]

  boot_disk {
    initialize_params {
      image = "ubuntu-os-cloud/ubuntu-2004-lts"
      size  = 10
    }
  }

  network_interface {
    subnetwork = google_compute_subnetwork.subnet.id
    # Conditionally assign external IP
    dynamic "access_config" {
      for_each = local.metrics_public ? [1] : []
      content {
        nat_ip = google_compute_address.metrics_ip[0].address
      }
    }
  }

  service_account {
    email  = local.service_account_email
    scopes = ["cloud-platform"]
  }

  metadata = {
    serial-port-enable = "true"  # Enable serial port logging
  }

  metadata_startup_script = <<-EOF
    #!/bin/bash
    
    # Set up logging
    exec 1> >(logger -s -t $(basename $0)) 2>&1
    echo "Starting startup script execution..."

    # Log metrics configuration
    echo "Metrics endpoint configuration:"
    %{if local.metrics_public}
    echo "Public metrics enabled. Endpoint IP: ${google_compute_address.metrics_ip[0].address}"
    %{else}
    echo "Metrics endpoint: local only"
    %{endif}

    # Install Google Cloud Ops Agent
    echo "Installing Google Cloud Ops Agent..."
    curl -sSO https://dl.google.com/cloudagents/add-google-cloud-ops-agent-repo.sh
    sudo bash add-google-cloud-ops-agent-repo.sh --also-install

    # Update and install basic dependencies
    echo "Installing basic dependencies..."
    apt-get update
    apt-get install -y git ca-certificates curl gnupg

    echo "Setting up Docker repository..."
    # Add Docker's official GPG key
    install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
    chmod a+r /etc/apt/keyrings/docker.gpg

    # Add Docker repository
    echo \
      "deb [arch="$(dpkg --print-architecture)" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
      "$(. /etc/os-release && echo "$VERSION_CODENAME")" stable" | \
      tee /etc/apt/sources.list.d/docker.list > /dev/null

    # Install Docker and Docker Compose
    echo "Installing Docker..."
    apt-get update
    apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

    # Create app directory and clone repo
    echo "Cloning repository..."
    mkdir -p /app
    git clone https://github.com/lgingerich/blockchain-indexer.git /app
    cd /app

    # Fetch config from Secret Manager and save it
    echo "Fetching config from Secret Manager..."
    gcloud secrets versions access latest --secret="indexer-${replace(local.chain_name, "_", "-")}-config" > /app/config.yml

    # Start Docker service
    echo "Starting Docker service..."
    systemctl start docker
    systemctl enable docker

    # Run your application with Docker Compose
    echo "Starting Docker Compose..."
    docker compose up -d

    echo "Startup script completed."
  EOF
}