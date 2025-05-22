use alloy_primitives::{Address, TxKind};
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::models::datasets::blocks::{RpcHeaderData, TransformedBlockData};
use crate::models::datasets::logs::{RpcLogReceiptData, TransformedLogData};
use crate::models::datasets::traces::{RpcTraceData, TransformedTraceData};
use crate::models::datasets::transactions::{
    RpcTransactionData, RpcTransactionReceiptData, TransformedTransactionData,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub chain_name: String,
    pub start_block: Option<u64>,
    pub end_block: Option<u64>,
    pub chain_tip_buffer: u64,
    pub rpc_url: String,
    pub dataset_location: String,
    pub datasets: Vec<String>,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Chain {
    Ethereum,
    ZKsync,
}

impl Chain {
    pub fn from_chain_id(chain_id: u64) -> Result<Self> {
        match chain_id {
            1 => Ok(Self::Ethereum),
            232 | 320 | 324 | 325 | 388 | 1217 | 1345 | 2741 | 2904 | 9637 | 50104 | 61166
            | 543210 => Ok(Self::ZKsync), // Lens | ZKcandy | ZKsync Era | GRVT | OpenZK | SxT | Cronos zkEVM | Abstract | Ripio LaChain | WonderFi |Sophon | Treasure Chain | Zero Network
            _ => Err(anyhow::anyhow!("Unsupported chain id: {}", chain_id)),
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
