use alloy_primitives::FixedBytes;
use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::HashMap;

use crate::models::{
    common::{ChainInfo, Schema},
    datasets::traces::{
        CommonTransformedTraceData, EthereumTransformedTraceData, RpcTraceData,
        TransformedTraceData, ZKsyncTransformedTraceData,
    },
};

pub trait TraceTransformer {
    fn transform_traces(
        traces: Vec<RpcTraceData>,
        chain_info: &ChainInfo,
        block_map: &HashMap<u64, (DateTime<Utc>, NaiveDate, FixedBytes<32>)>,
        tx_index_map: &HashMap<FixedBytes<32>, Option<u64>>,
    ) -> Result<Vec<TransformedTraceData>>;
}

impl TraceTransformer for RpcTraceData {
    fn transform_traces(
        traces: Vec<RpcTraceData>,
        chain_info: &ChainInfo,
        block_map: &HashMap<u64, (DateTime<Utc>, NaiveDate, FixedBytes<32>)>,
        tx_index_map: &HashMap<FixedBytes<32>, Option<u64>>,
    ) -> Result<Vec<TransformedTraceData>> {
        traces
            .into_iter()
            .map(|trace| {
                let common_data = match &trace {
                    RpcTraceData::Ethereum(t) => &t.common,
                    RpcTraceData::ZKsync(t) => &t.common,
                };

                let tx_hash = common_data
                    .tx_hash
                    .ok_or_else(|| anyhow::anyhow!("Missing tx_hash for trace primary key"))?;

                let pk = if common_data.trace_address.is_empty() {
                    format!(
                        "trace_{}_{}_{}",
                        chain_info.id, tx_hash, common_data.trace_type
                    )
                } else {
                    format!(
                        "trace_{}_{}_{}_{}",
                        chain_info.id,
                        tx_hash,
                        common_data.trace_type,
                        common_data
                            .trace_address
                            .iter()
                            .map(|&x| x.to_string())
                            .collect::<Vec<String>>()
                            .join("_")
                    )
                };

                let common = CommonTransformedTraceData {
                    id: pk,
                    chain_id: chain_info.id,
                    block_time: block_map
                        .get(&common_data.block_number)
                        .map(|(time, _, _)| *time)
                        .unwrap_or_default(),
                    block_date: block_map
                        .get(&common_data.block_number)
                        .map(|(_, date, _)| *date)
                        .unwrap_or_default(),
                    block_number: common_data.block_number,
                    block_hash: block_map
                        .get(&common_data.block_number)
                        .map(|(_, _, hash)| *hash)
                        .unwrap_or_default(),
                    tx_hash: Some(tx_hash),
                    tx_index: common_data
                        .tx_hash
                        .and_then(|hash| tx_index_map.get(&hash).copied().flatten()),
                    trace_type: common_data.trace_type.clone(),
                    subtraces: common_data.subtraces,
                    trace_address: common_data.trace_address.clone(),
                    from_address: common_data.from_address,
                    to_address: common_data.to_address,
                    value: common_data.value.clone(),
                    gas: common_data.gas.clone(),
                    gas_used: common_data.gas_used.clone(),
                    input: common_data.input.clone(),
                    output: common_data.output.clone(),
                    error: common_data.error.clone(),
                };

                match chain_info.schema {
                    Schema::Ethereum => Ok(TransformedTraceData::Ethereum(
                        EthereumTransformedTraceData { common },
                    )),
                    Schema::ZKsync => {
                        Ok(TransformedTraceData::ZKsync(ZKsyncTransformedTraceData {
                            common,
                        }))
                    }
                }
            })
            .collect::<Result<Vec<_>>>()
    }
}
