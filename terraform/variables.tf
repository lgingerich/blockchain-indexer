variable "project_id" {
  description = "GCP Project ID"
  type        = string
}

variable "region" {
  description = "GCP Region"
  type        = string
  default     = "us-central1"
}

variable "zone" {
  description = "GCP Zone"
  type        = string
  default     = "us-central1-a"
}

variable "machine_type" {
  description = "GCP Machine Type"
  type        = string
  default     = "e2-medium"
}

variable "chain_name" {
  description = "Name of the chain"
  type        = string
}

variable "git_repo_url" {
  description = "URL of the git repository to clone"
  type        = string
}