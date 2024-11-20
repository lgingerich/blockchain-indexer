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

variable "create_service_account" {
  description = "Whether to create a new service account or use existing one"
  type        = bool
  default     = false  # Default to using existing service account
}

variable "chain_name" {
  description = "Name of the chain"
  type        = string
}

variable "git_repo_url" {
  description = "URL of the git repository to clone"
  type        = string
}