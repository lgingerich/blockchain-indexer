pub mod retry;

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use config::{Config, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use tracing::warn;

use crate::models::common::Config as IndexerConfig;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Table {
    Blocks,
    Logs,
    Transactions,
    Traces,
}

impl Table {
    pub fn from_string(table: &String) -> Result<Table> {
        match table.as_str() {
            "blocks" => Ok(Table::Blocks),
            "logs" => Ok(Table::Logs),
            "transactions" => Ok(Table::Transactions),
            "traces" => Ok(Table::Traces),
            _ => Err(anyhow::anyhow!("'{}' is not a valid table name", table)),
        }
    }

    pub fn from_vec(tables: Vec<String>) -> Result<Vec<Table>> {
        tables.iter().map(Self::from_string).collect()
    }

    #[allow(dead_code)]
    pub fn to_vec(tables: &[Table]) -> Vec<String> {
        tables.iter().map(|t| t.to_string()).collect()
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Table::Blocks => write!(f, "blocks"),
            Table::Logs => write!(f, "logs"),
            Table::Transactions => write!(f, "transactions"),
            Table::Traces => write!(f, "traces"),
        }
    }
}

pub fn load_config<P: AsRef<Path>>(file_path: P) -> Result<IndexerConfig> {
    let settings = Config::builder()
        .add_source(File::from(file_path.as_ref()).format(FileFormat::Yaml))
        .build()?;

    let mut cfg: IndexerConfig = settings.try_deserialize()?;

    // Convert hyphens to underscores in all relevant fields
    cfg.chain_name = cfg.chain_name.replace('-', "_");

    Ok(cfg)
}

// TODO: Refactor so I don't need multiple conversion functions
pub fn hex_to_u64(hex: String) -> Result<u64> {
    Ok(u64::from_str_radix(hex.trim_start_matches("0x"), 16)?)
}

pub fn hex_to_u128(hex: String) -> Result<u128> {
    Ok(u128::from_str_radix(hex.trim_start_matches("0x"), 16)?)
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
pub fn sanitize_block_time(block_number: u64, datetime: DateTime<Utc>) -> Result<DateTime<Utc>> {
    // If this is block 0 with a Unix epoch date (or very close to it)
    if block_number == 0 && datetime.format("%Y").to_string() == "1970" {
        // Use January 1, 2020 as the fallback date
        let fallback_date = NaiveDate::from_ymd_opt(2020, 1, 1)
            .ok_or_else(|| anyhow::anyhow!("Internal error: Hardcoded fallback date (2020-01-01) for block time sanitization is invalid."))?;
        let fallback_time = datetime.time();
        let fallback_datetime =
            DateTime::<Utc>::from_naive_utc_and_offset(fallback_date.and_time(fallback_time), Utc);

        warn!(
            "Sanitized block 0 time from {} (Unix epoch) to {} to avoid BigQuery partitioning errors",
            datetime, fallback_datetime
        );
        Ok(fallback_datetime)
    } else {
        Ok(datetime)
    }
}
