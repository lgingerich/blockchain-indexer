// Temporary disable warnings for development
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use eyre::Result;

use crate::models::indexed::transactions::TransformedTransactionData;
use crate::models::common::ParsedData;

pub trait TransactionTransformer {
    fn transform_transactions(self) -> Result<Vec<TransformedTransactionData>>;
}



// // TODO ############################################################


// // TODO: Confirm I want all these fields
// impl TransactionTransformer for ParsedData {
//     fn transform_transactions(self) -> Result<Vec<TransformedTransactionData>> {
//         Ok(vec![TransformedTransactionData {

//         }])
//     }
// }