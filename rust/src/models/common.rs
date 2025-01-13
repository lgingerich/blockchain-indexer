use alloy_primitives::{Address, TxKind};
use serde::{Deserialize, Serialize};

use crate::indexer::rpc::blocks::BlockParser;
use crate::indexer::rpc::receipts::ReceiptParser;

use crate::models::datasets::blocks::{RpcHeaderData, RpcWithdrawalData, TransformedBlockData};
use crate::models::datasets::logs::{RpcLogReceiptData, TransformedLogData};
use crate::models::datasets::transactions::{RpcTransactionData, RpcTransactionReceiptData, TransformedTransactionData};
use crate::models::datasets::traces::{RpcTraceData, TransformedTraceData};


#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub project_name: String,
    pub chain_name: String,
    pub chain_schema: String,
    pub rpc_url: String,
    pub datasets: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum TransactionTo {
    TxKind(TxKind),   // For TxLegacy, TxEip2930, TxEip1559 which use TxKind
    Address(Address), // For TxEip4844, TxEip7702 which use Address directly
}

#[derive(Debug, Clone)]
pub struct ParsedData {
    pub chain_id: u64,
    pub header: Vec<RpcHeaderData>,
    pub transactions: Vec<RpcTransactionData>,
    pub withdrawals: Vec<RpcWithdrawalData>,
    pub transaction_receipts: Vec<RpcTransactionReceiptData>,
    pub logs: Vec<RpcLogReceiptData>,
    pub traces: Vec<RpcTraceData>,
}

#[derive(Debug)]
pub struct TransformedData {
    pub blocks: Vec<TransformedBlockData>,
    pub transactions: Vec<TransformedTransactionData>,
    pub logs: Vec<TransformedLogData>,
    pub traces: Vec<TransformedTraceData>,
}
