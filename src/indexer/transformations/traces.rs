use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::HashMap;

use crate::models::datasets::traces::{
    CommonTransformedTraceData, EthereumTransformedTraceData, RpcTraceData, TransformedTraceData,
    ZKsyncTransformedTraceData,
};
use crate::models::common::Chain;

pub trait TraceTransformer {
    fn transform_traces(
        traces: Vec<RpcTraceData>,
        chain: Chain,
        chain_id: u64,
        block_map: &HashMap<u64, (DateTime<Utc>, NaiveDate)>,
    ) -> Result<Vec<TransformedTraceData>>;
}

impl TraceTransformer for RpcTraceData {
    fn transform_traces(
        traces: Vec<RpcTraceData>,
        chain: Chain,
        chain_id: u64,
        block_map: &HashMap<u64, (DateTime<Utc>, NaiveDate)>,
    ) -> Result<Vec<TransformedTraceData>> {
        Ok(traces
            .into_iter()
            .map(|trace| {
                let common_data = match &trace {
                    RpcTraceData::Ethereum(t) => &t.common,
                    RpcTraceData::ZKsync(t) => &t.common,
                };

                let common = CommonTransformedTraceData {
                    chain_id,
                    block_time: block_map
                        .get(&common_data.block_number)
                        .map(|(time, _)| *time)
                        .unwrap_or_default(),
                    block_date: block_map
                        .get(&common_data.block_number)
                        .map(|(_, date)| *date)
                        .unwrap_or_default(),
                    block_number: common_data.block_number,
                    tx_hash: common_data.tx_hash,
                    r#type: common_data.r#type.clone(),
                    trace_address: common_data.trace_address.clone(),
                    from: common_data.from,
                    to: common_data.to,
                    value: common_data.value.clone(),
                    gas: common_data.gas.clone(),
                    gas_used: common_data.gas_used.clone(),
                    input: common_data.input.clone(),
                    output: common_data.output.clone(),
                    error: common_data.error.clone(),
                    revert_reason: common_data.revert_reason.clone(),
                    logs: common_data.logs.clone(),
                };

                match chain {
                    Chain::Ethereum => TransformedTraceData::Ethereum(EthereumTransformedTraceData { common }),
                    Chain::ZKsync => TransformedTraceData::ZKsync(ZKsyncTransformedTraceData { common }),
                }
            })
            .collect())
    }
}

