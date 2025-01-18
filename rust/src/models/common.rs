use alloy_primitives::{Address, TxKind};
use serde::{Deserialize, Serialize};

use crate::indexer::rpc::blocks::BlockParser;
use crate::indexer::rpc::receipts::ReceiptParser;

use crate::models::datasets::blocks::{RpcHeaderData, TransformedBlockData};
use crate::models::datasets::logs::{RpcLogReceiptData, TransformedLogData};
use crate::models::datasets::traces::{RpcTraceData, TransformedTraceData};
use crate::models::datasets::transactions::{
    RpcTransactionData, RpcTransactionReceiptData, TransformedTransactionData,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub project_name: String,
    pub chain_name: String,
    pub rpc_url: String,
    pub datasets: Vec<String>,
    pub chain_id: u64,
    pub chain_tip_buffer: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Chain {
    Ethereum,
    ZKsync,
}

impl Chain {
    pub fn from_chain_id(chain_id: u64) -> Self {
        match chain_id {
            1 => Self::Ethereum,
            324 => Self::ZKsync,    // ZKsync Era
            325 => Self::ZKsync,    // GRVT
            388 => Self::ZKsync,    // Cronos zkEVM
            50104 => Self::ZKsync,  // Sophon
            61166 => Self::ZKsync,  // Treasury Chain
            543210 => Self::ZKsync, // Zero Network
            _ => Self::ZKsync,      // Default to ZKsync for unknown chains
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TransactionTo {
    TxKind(TxKind),   // For TxLegacy, TxEip2930, TxEip1559 which use TxKind
    Address(Address), // For TxEip4844, TxEip7702 which use Address directly
}

#[derive(Debug, Clone)]
pub struct ParsedData {
    pub chain_id: u64,
    pub header: Vec<RpcHeaderData>,
    pub transactions: Vec<RpcTransactionData>,
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
