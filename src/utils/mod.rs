pub mod rate_limiter;
pub mod retry;

use anyhow::{Context, Result};
use std::{fs, path::Path};
use tracing::info;

use crate::models::common::Config;

pub fn hex_to_u64(hex: String) -> Option<u64> {
    u64::from_str_radix(hex.trim_start_matches("0x"), 16).ok()
}

pub fn load_config<P: AsRef<Path>>(file_name: P) -> Result<Config> {
    // Build the path to the config file
    let manifest_dir = env!("CARGO_MANIFEST_DIR").to_string();
    let config_path = Path::new(&manifest_dir).join(file_name);
    info!("Config path: {}", config_path.to_string_lossy());

    // Read the file contents to a string
    let contents = fs::read_to_string(config_path).context("failed to read config file")?;

    // Parse the YAML into our Config struct
    let mut config: Config =
        serde_yaml::from_str(&contents).context("failed to parse config YAML")?;

    // Convert hyphens to underscores in all relevant fields
    config.chain_name = config.chain_name.replace('-', "_");

    Ok(config)
}

fn strip_html(error: &str) -> String {
    // If the error contains HTML tags, extract just the text content
    if error.contains("<!doctype html>") || error.contains("<html>") {
        // Remove all HTML tags and return the first non-empty line of text
        error
            .lines()
            .map(|line| line.trim())
            .find(|line| {
                !line.starts_with('<')
                    && !line.ends_with('>')
                    && !line.is_empty()
                    && !line.starts_with("<!")
                    && *line != "html"
                    && *line != "body"
            })
            .unwrap_or(error)
            .to_string()
    } else {
        // Return original error if no HTML
        error.to_string()
    }
}
