use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockError {
    #[error("Invalid block format: Expected full transaction objects but received only transaction hashes")]
    TransactionHashesOnly,
    #[error("Invalid block type: Uncle/ommer blocks are not supported by this indexer")]
    UncleBlocksNotSupported,
}

#[derive(Error, Debug)]
pub enum ReceiptError {
    #[error("Missing required field in receipt: {field}")]
    MissingField { field: String },
    
    #[error("Invalid timestamp value {value} in log receipt")]
    InvalidTimestamp { value: i64 },
}
