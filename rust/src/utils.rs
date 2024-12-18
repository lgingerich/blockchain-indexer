use eyre::{Result, WrapErr};
use tracing::info;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;
use std::path::Path;
use tokio::time::{sleep, Duration};

use crate::models::common::Config;

// Constants for retry configuration
const MAX_RETRIES: u32 = 5;
const BASE_DELAY_MS: u64 = 100;
const MAX_DELAY_MS: u64 = 5000;

/// Implements exponential backoff with jitter
pub async fn exponential_backoff(attempt: u32, max_retries: u32) {
    let max_shift = std::cmp::min(MAX_DELAY_MS, BASE_DELAY_MS * (1 << attempt));
    let jitter = rand::thread_rng().gen_range(0..=50);
    let delay = Duration::from_millis(max_shift + jitter);
    sleep(delay).await;
}

pub fn load_config<P: AsRef<Path>>(file_name: P) -> Result<Config> {
    // Build the path to the config file
    let manifest_dir = env!("CARGO_MANIFEST_DIR").to_string();
    let config_path = Path::new(&manifest_dir).join(file_name);
    info!("Config path: {}", config_path.to_string_lossy());

    // Read the file contents to a string
    let contents = fs::read_to_string(config_path)
        .wrap_err("failed to read config file")?;
    
    // Parse the YAML into our Config struct
    let mut config: Config = serde_yaml::from_str(&contents)
        .wrap_err("failed to parse config YAML")?;

    // Convert hyphens to underscores in all relevant fields
    config.project_name = config.project_name.replace('-', "_");
    config.chain_name = config.chain_name.replace('-', "_");
    config.chain_schema = config.chain_schema.replace('-', "_");
    
    Ok(config)
}

