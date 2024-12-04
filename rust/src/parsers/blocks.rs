use alloy_rpc_types_eth::{Block, Header, Withdrawal};
use eyre::Result;

pub trait BlockParser {
    fn parse_raw(self) -> Result<Block>;
}

impl BlockParser for Block {
    fn parse_raw(self) -> Result<Block> {
        Ok(self)
    }
}


// pub trait BlockParser {
//     // fn parse_header(self) -> Result<Header>;
//     // fn parse_uncles(self) -> Result<Vec<String>>;
//     // fn parse_withdrawals(self) -> Result<Option<Vec<Withdrawal>>>;
//     fn parse_raw(self) -> Result<Block>;
// }

// impl BlockParser for Block {
//     fn parse_raw(self) -> Result<Block> {
//         Ok(self)
//     }
    
//     // fn parse_header(self) -> Result<Header> {
//     //     Ok(Header {
//     //         hash: self.hash.unwrap_or_default(),
//     //         inner: self,
//     //         total_difficulty: self.total_difficulty,
//     //         size: self.size,
//     //     })
//     // }

//     // fn parse_uncles(self) -> Result<Vec<String>> {
//     //     Ok(self.uncles
//     //         .into_iter()
//     //         .map(|u| format!("{:?}", u))
//     //         .collect())
//     // }

//     // fn parse_withdrawals(self) -> Result<Option<Vec<Withdrawal>>> {
//     //     Ok(self.withdrawals.map(|w| w.into_iter().collect()))
//     // }
// }