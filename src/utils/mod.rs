pub mod retry;

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use std::{fs, path::Path};
use tracing::{info, warn};

use crate::models::common::Config;

// TODO: Refactor so I don't need multiple conversion functions
pub fn hex_to_u64(hex: String) -> Option<u64> {
    u64::from_str_radix(hex.trim_start_matches("0x"), 16).ok()
}

pub fn hex_to_u128(hex: String) -> Option<u128> {
    u128::from_str_radix(hex.trim_start_matches("0x"), 16).ok()
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

/// Strips HTML content from API error responses, with special handling for HTTP status codes
///
/// # Arguments
/// * `response` - The API response string that may contain HTML
///
/// # Returns
/// * `String` - The cleaned string with HTML removed and status codes preserved
pub fn strip_html(response: &str) -> String {
    // Define the state for our parser
    enum State {
        Normal,
        InTag,
        InScript,
        InStyle,
    }

    let mut result = String::with_capacity(response.len());
    let mut state = State::Normal;

    // Process each character in the response
    let mut chars = response.chars().peekable();
    while let Some(c) = chars.next() {
        match state {
            State::Normal => {
                if c == '<' {
                    // Check if we're entering a script or style tag
                    let tag: String = chars
                        .clone()
                        .take_while(|&c| c != '>' && c != ' ')
                        .collect();

                    if tag.eq_ignore_ascii_case("script") {
                        state = State::InScript;
                    } else if tag.eq_ignore_ascii_case("style") {
                        state = State::InStyle;
                    } else {
                        state = State::InTag;
                    }
                } else {
                    // Preserve non-HTML content
                    result.push(c);
                }
            }
            State::InTag => {
                if c == '>' {
                    // Add a space to preserve text formatting
                    result.push(' ');
                    state = State::Normal;
                }
            }
            State::InScript => {
                // Look for </script> to exit script state
                if c == '<' && chars.peek() == Some(&'/') {
                    // Check for "script>"
                    let script_close: String = chars
                        .clone()
                        .skip(1) // Skip the '/'
                        .take_while(|&c| c != '>')
                        .collect();

                    if script_close.eq_ignore_ascii_case("script") {
                        // Skip past the "</script>"
                        for _ in 0.."script>".len() {
                            chars.next();
                        }
                        state = State::Normal;
                    }
                }
            }
            State::InStyle => {
                // Look for </style> to exit style state
                if c == '<' && chars.peek() == Some(&'/') {
                    // Check for "style>"
                    let style_close: String = chars
                        .clone()
                        .skip(1) // Skip the '/'
                        .take_while(|&c| c != '>')
                        .collect();

                    if style_close.eq_ignore_ascii_case("style") {
                        // Skip past the "</style>"
                        for _ in 0.."style>".len() {
                            chars.next();
                        }
                        state = State::Normal;
                    }
                }
            }
        }
    }

    // Normalize and clean up the response
    let result = result.trim().to_string();

    // Special handling for HTTP status codes in the format you're seeing
    if let Some(status_code) = extract_http_status(&result) {
        return format!("HTTP {}: {}", status_code, result.trim());
    }

    result
}

/// Extract HTTP status code from a string if present
fn extract_http_status(s: &str) -> Option<u16> {
    // Common pattern in HTTP error responses: number followed by text
    let s = s.trim();

    // Look for patterns like "429 Too Many Requests" or "429 429 Too Many Requests"
    let mut words = s.split_whitespace();
    if let Some(first_word) = words.next() {
        if let Ok(status) = first_word.parse::<u16>() {
            if status >= 100 && status < 600 {
                // Valid HTTP status range
                return Some(status);
            }
        }
    }

    None
}

// For testing the function
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html() {
        // Basic HTML
        let html = "<html><body>Hello world</body></html>";
        assert_eq!(strip_html(html), "Hello world");

        // With HTTP status code
        let html = "<html><body>429 429 Too Many Requests</body></html>";
        assert_eq!(strip_html(html), "HTTP 429: 429 Too Many Requests");

        // Real-world example from logs
        let html = "    429 429 Too Many Requests";
        assert_eq!(strip_html(html), "HTTP 429: 429 429 Too Many Requests");
    }
}
