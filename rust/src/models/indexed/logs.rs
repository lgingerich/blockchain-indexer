// use alloy_primitives::{Address, Bloom, Bytes, FixedBytes, TxKind, Uint};
// use chrono::{DateTime, NaiveDate, Utc};
// use crate::models::common::{ChainId, TransactionTo};


// #[derive(Debug)]
// pub struct TransformedLogData {
//     pub address: Address,
//     pub topics: Vec<FixedBytes<32>>,
//     pub data: Bytes,
//     pub block_hash: Option<FixedBytes<32>>,
//     pub block_number: Option<u64>,
//     pub block_timestamp: Option<u64>,
//     pub transaction_hash: Option<FixedBytes<32>>,
//     pub transaction_index: Option<u64>,
//     pub log_index: Option<u64>,
//     pub removed: bool
// }




use crate::models::rpc::receipts::LogReceiptData;

// Create type alias for LogReceiptData type
// Do not expect to need custom modifications to this type
pub type TransformedLogData = LogReceiptData;