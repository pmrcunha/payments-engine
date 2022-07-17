use std::{error::Error, fmt};

#[derive(Debug)]
pub enum TransactionErrorType {
    NoDepositAmount,
    NoWithdrawalAmount
}

#[derive(Debug)]
pub struct TransactionRecordError {
    pub error_type: TransactionErrorType
}

impl Error for TransactionRecordError {}

impl fmt::Display for TransactionRecordError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.error_type {
            TransactionErrorType::NoDepositAmount => write!(f, "A deposit must have an amount"),
            TransactionErrorType::NoWithdrawalAmount => write!(f, "An withdrawal must have an amount")
        }
        
    }
}