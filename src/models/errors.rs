use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChainError {
    #[error("Unsupported chain ID: {chain_id}")]
    UnsupportedChainId { chain_id: u64 },
}

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("Invalid block number response: expected number, got {got}")]
    InvalidBlockNumberResponse { got: String },
}

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
}
