pub mod retry;

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use std::{fs, path::Path};
use tracing::{info, warn};

use crate::models::common::Config;

pub fn hex_to_u64(hex: String) -> Option<u64> {
    u64::from_str_radix(hex.trim_start_matches("0x"), 16).ok()
}

// Sanitizes block dates for block 0 to avoid BigQuery partitioning errors.
// BigQuery only supports partitioning up to 3650 days in the past (i.e. 10 years).
// If block 0 has a date in 1970 (Unix epoch), replaces it with January 1, 2020.
// January 1, 2020 is an arbitrary date which will allow this indexer to work properly
// until the year 2030.
// NOTE: This indexer with BigQuery should not be used for Ethereum. While it could work
// if the sanitized date was set to 2015 when Ethereum was launched, that will soon be
// exceeded and then the indexer will fail for all chains. As this is tailored towards
// usage with L2s, we will focus on safe usage with L2s.

pub fn sanitize_block_time(block_number: u64, datetime: DateTime<Utc>) -> DateTime<Utc> {
    // If this is block 0 with a Unix epoch date (or very close to it)
    if block_number == 0 && datetime.format("%Y").to_string() == "1970" {
        // Use January 1, 2020 as the fallback date
        let fallback_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let fallback_time = datetime.time();
        let fallback_datetime =
            DateTime::<Utc>::from_naive_utc_and_offset(fallback_date.and_time(fallback_time), Utc);

        warn!(
            "Sanitized block 0 time from {} (Unix epoch) to {} to avoid BigQuery partitioning errors",
            datetime, fallback_datetime
        );
        fallback_datetime
    } else {
        datetime
    }
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
