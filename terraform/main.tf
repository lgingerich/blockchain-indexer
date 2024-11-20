terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 4.0"
    }
  }
}

provider "google" {
  project = var.project_id
  region  = var.region
  zone    = var.zone
}

# Try to get existing service account
data "google_service_account" "existing_vm_sa" {
  count      = var.create_service_account ? 0 : 1
  account_id = "indexer-vm-sa"
  project    = var.project_id
}

# Create service account if it doesn't exist
resource "google_service_account" "vm_service_account" {
  count        = var.create_service_account ? 1 : 0
  account_id   = "indexer-vm-sa"
  display_name = "Indexer VM Service Account"
  project      = var.project_id
}

# Use a local to simplify the email reference
locals {
  service_account_email = var.create_service_account ? (
    google_service_account.vm_service_account[0].email
  ) : data.google_service_account.existing_vm_sa[0].email
}

# Grant BigQuery permissions to the service account
resource "google_project_iam_member" "bigquery_access" {
  project = var.project_id
  role    = "roles/bigquery.dataEditor"
  member  = "serviceAccount:${local.service_account_email}"
}

# Grant BigQuery Job User permissions to the service account
resource "google_project_iam_member" "bigquery_job_user" {
  project = var.project_id
  role    = "roles/bigquery.jobUser"
  member  = "serviceAccount:${local.service_account_email}"
}

# Grant Cloud Logging permissions to the service account
resource "google_project_iam_member" "logging_access" {
  project = var.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${local.service_account_email}"
}

# VPC network
resource "google_compute_network" "vpc_network" {
  name                    = "indexer-${var.chain_name}-network"
  auto_create_subnetworks = false
}

# Subnet
resource "google_compute_subnetwork" "subnet" {
  name          = "indexer-${var.chain_name}-subnet"
  ip_cidr_range = "10.0.0.0/24"
  network       = google_compute_network.vpc_network.id
  region        = var.region
}

# IAP SSH firewall rule
resource "google_compute_firewall" "iap_ssh" {
  name    = "allow-iap-ssh-${var.chain_name}"
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
  name    = "indexer-${var.chain_name}-router"
  region  = var.region
  network = google_compute_network.vpc_network.id
}

# Add Cloud NAT config
resource "google_compute_router_nat" "nat" {
  name                               = "indexer-${var.chain_name}-nat"
  router                             = google_compute_router.router.name
  region                             = var.region
  nat_ip_allocate_option             = "AUTO_ONLY"
  source_subnetwork_ip_ranges_to_nat = "ALL_SUBNETWORKS_ALL_IP_RANGES"
}

# VM Instance
resource "google_compute_instance" "indexer_vm" {
  name         = "indexer-${var.chain_name}"
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
    # No external IP
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
    git clone ${var.git_repo_url} /app
    cd /app

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