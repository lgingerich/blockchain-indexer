use alloy_primitives::{Address, TxKind};
use serde::Serialize;

use crate::indexer::rpc::blocks::BlockParser;
use crate::indexer::rpc::receipts::ReceiptParser;
use crate::models::indexed::blocks::TransformedBlockData;
use crate::models::indexed::logs::TransformedLogData;
use crate::models::indexed::traces::TransformedTraceData;
use crate::models::indexed::transactions::TransformedTransactionData;
use crate::models::rpc::blocks::{HeaderData, TransactionData, WithdrawalData};
use crate::models::rpc::receipts::{LogReceiptData, TransactionReceiptData};
use crate::models::rpc::traces::TraceData;

#[derive(Debug, Clone, Serialize)]
pub enum ChainId {
    Legacy(Option<u64>), // For TxLegacy where chain_id is Option<u64>
    Other(u64),          // For all other tx types where chain_id is u64
}

#[derive(Debug, Clone, Serialize)]
pub enum TransactionTo {
    TxKind(TxKind),   // For TxLegacy, TxEip2930, TxEip1559 which use TxKind
    Address(Address), // For TxEip4844, TxEip7702 which use Address directly
}

// TODO: The user will be able to specify which datasets they want, so these should all be optional
#[derive(Debug, Clone)]
pub struct ParsedData {
    pub chain_id: u64,
    pub header: HeaderData,
    pub transactions: Vec<TransactionData>,
    pub withdrawals: Vec<WithdrawalData>,
    pub transaction_receipts: Vec<TransactionReceiptData>,
    pub logs: Vec<LogReceiptData>,
    pub traces: Vec<TraceData>,
}

// TODO: The user will be able to specify which datasets they want, so these should all be optional
#[derive(Debug)]
pub struct TransformedData {
    pub blocks: Vec<TransformedBlockData>,
    pub transactions: Vec<TransformedTransactionData>,
    pub logs: Vec<TransformedLogData>,
    pub traces: Vec<TransformedTraceData>,
}
