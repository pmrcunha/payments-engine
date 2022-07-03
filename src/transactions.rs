use core::fmt;
use std::convert::TryFrom;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    pub amount: Option<f32>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(try_from = "String")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl TryFrom<String> for TransactionType {
    type Error = TransactionTypeFromStrError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "deposit" => Ok(TransactionType::Deposit),
            "withdrawal" => Ok(TransactionType::Withdrawal),
            "dispute" => Ok(TransactionType::Dispute),
            "resolve" => Ok(TransactionType::Resolve),
            "chargeback" => Ok(TransactionType::Chargeback),
            _ => Err(TransactionTypeFromStrError),
        }
    }
}

pub struct TransactionTypeFromStrError;

impl fmt::Display for TransactionTypeFromStrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Could not decode CSV type into the transaction type enum")
    }
}