use alloy_primitives::{Address, TxKind};


use crate::indexer::rpc::blocks::BlockParser;
use crate::indexer::rpc::receipts::ReceiptParser;
use crate::models::rpc::blocks::{HeaderData, TransactionData, WithdrawalData};
use crate::models::rpc::receipts::{LogReceiptData, TransactionReceiptData};
use crate::models::indexed::blocks::TransformedBlockData;
use crate::models::indexed::transactions::TransformedTransactionData;
use crate::models::indexed::logs::TransformedLogData;


#[derive(Debug, Clone)]
pub enum ChainId {
    Legacy(Option<u64>),  // For TxLegacy where chain_id is Option<u64>
    Other(u64)           // For all other tx types where chain_id is u64
}

#[derive(Debug)]
pub enum TransactionTo {
    TxKind(TxKind),      // For TxLegacy, TxEip2930, TxEip1559 which use TxKind
    Address(Address),    // For TxEip4844, TxEip7702 which use Address directly
}

#[derive(Debug)]
pub struct ParsedData {
    pub header: HeaderData,
    pub transactions: Vec<TransactionData>,
    pub withdrawals: Vec<WithdrawalData>,
    pub transaction_receipts: Vec<TransactionReceiptData>,
    pub logs: Vec<LogReceiptData>
}

#[derive(Debug)]
pub struct TransformedData {
    pub blocks: Vec<TransformedBlockData>,
    // pub transactions: Vec<TransformedTransactionData>,
    // pub logs: Vec<TransformedLogData>
}