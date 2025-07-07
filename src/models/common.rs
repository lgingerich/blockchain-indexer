use alloy_primitives::{Address, TxKind};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::models::datasets::{
    blocks::{RpcHeaderData, TransformedBlockData},
    logs::{RpcLogReceiptData, TransformedLogData},
    traces::{RpcTraceData, TransformedTraceData},
    transactions::{RpcTransactionData, RpcTransactionReceiptData, TransformedTransactionData},
};

static CHAIN_INFO: OnceLock<ChainInfo> = OnceLock::new();

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Schema {
    Ethereum,
    ZKsync,
}

impl Schema {
    pub fn from_chain_id(chain_id: u64) -> Result<Self> {
        match chain_id {
            1 => Ok(Self::Ethereum),
            232 | 320 | 324 | 325 | 388 | 1217 | 1345 | 2741 | 2904 | 9075 | 9637 | 50104
            | 61166 | 543210 => Ok(Self::ZKsync), // Lens | ZKcandy | ZKsync Era | GRVT | OpenZK | SxT | Cronos zkEVM | Abstract | Ripio LaChain | WonderFi | Gateway | Sophon | Treasure Schema | Zero Network
            _ => Ok(Self::Ethereum) // Default to Ethereum for unknown chains
        }
    }
}

impl std::fmt::Display for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Schema::Ethereum => write!(f, "Ethereum"),
            Schema::ZKsync => write!(f, "ZKsync"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChainInfo {
    pub id: u64,
    pub name: String,
    pub schema: Schema,
}

impl ChainInfo {
    pub fn new(id: u64, name: String, schema: Schema) -> Self {
        Self { id, name, schema }
    }

    pub fn set_chain_info(chain_info: Self) {
        let _ = CHAIN_INFO.set(chain_info);
    }

    pub fn get_chain_info() -> &'static ChainInfo {
        CHAIN_INFO.get().expect("CHAIN_INFO must be initialized before use")
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TransactionTo {
    TxKind(TxKind),   // For TxLegacy, TxEip2930, TxEip1559 which use TxKind
    Address(Address), // For TxEip4844, TxEip7702 which use Address directly
}

#[derive(Debug, Clone)]
pub struct ParsedData {
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
