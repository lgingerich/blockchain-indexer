use anyhow::Result;

use crate::models::common::{Chain, ParsedData};
use crate::models::datasets::traces::{
    CommonTransformedTraceData, EthereumTransformedTraceData, RpcTraceData, TransformedTraceData,
    ZKsyncTransformedTraceData,
};

pub trait TraceTransformer {
    fn transform_traces(self, chain: Chain) -> Result<Vec<TransformedTraceData>>;
}

impl TraceTransformer for ParsedData {
    fn transform_traces(self, chain: Chain) -> Result<Vec<TransformedTraceData>> {
        Ok(self
            .traces
            .into_iter()
            .map(|trace| {
                // First match on the log to get the common data
                let common_data = match &trace {
                    RpcTraceData::Ethereum(t) => &t.common,
                    RpcTraceData::ZKsync(t) => &t.common,
                };

                let common = CommonTransformedTraceData {
                    chain_id: self.chain_id,
                    from: common_data.from,
                    gas: common_data.gas,
                    gas_used: common_data.gas_used,
                    to: common_data.to,
                    input: common_data.input.clone(),
                    output: common_data.output.clone(),
                    error: common_data.error.clone(),
                    revert_reason: common_data.revert_reason.clone(),
                    logs: common_data.logs.clone(),
                    value: common_data.value,
                    typ: common_data.typ.clone(),
                };

                match chain {
                    Chain::Ethereum => {
                        TransformedTraceData::Ethereum(EthereumTransformedTraceData { common })
                    }
                    Chain::ZKsync => {
                        TransformedTraceData::ZKsync(ZKsyncTransformedTraceData { common })
                    }
                }
            })
            .collect())
    }
}
