mod transactions;
mod accounts;
mod custom_errors;

use std::collections::HashMap;
use std::collections::btree_map::{BTreeMap, Entry};
use std::env;
use std::error::Error;
use std::path::Path;
use std::process;
use transactions::{Transaction, TransactionType};
use accounts::AccountBalance;
use custom_errors::{TransactionRecordError, TransactionErrorType};

/// Takes the path to a CSV file with transactions and outputs 
/// the account balances.
fn process_csv(path: &Path) -> Result<String, Box<dyn Error>> {
    // We keep a map of the account balances throughout the whole execution of the program.
    // This is because we can get an update to a given client balance all the way to the last
    // transaction, and we only want to output the results once, at the end.
    // We use a BTreeMap because we want to display sorted results.
    let mut account_balances: BTreeMap<u16, AccountBalance> = BTreeMap::new();

    // We hold a record of the deposit transaction amounts, so that we can process disputes
    let mut deposit_transaction_amounts: HashMap<u32, f32> = HashMap::new();
    // We hold a record of the disputed transactions, since resolves and chargebacks are only valid for those
    let mut disputed_transactions: Vec<u32> = vec![];

    // Setup a reader from the given path to a CSV file.
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b',')
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(path)?;

    for transaction_record in rdr.deserialize() {
        let transaction: Transaction = transaction_record?;

        let account_balance = match account_balances.entry(transaction.client_id) {
            // If the client already exists, get its balance
            Entry::Occupied(e) => e.into_mut(),
            // If the client doesn't exist, insert it and get a new balance
            Entry::Vacant(e) => {
                e.insert(AccountBalance {
                    client: transaction.client_id,
                    available: 0.0,
                    held: 0.0,
                    locked: false,
                })
            }
        };

        if account_balance.locked {
            continue;
        }

        match transaction.tx_type {
            TransactionType::Deposit => {
                // Handle a deposit
                if let Some(amount) = transaction.amount {
                    account_balance.available += amount;
                    deposit_transaction_amounts.insert(transaction.tx_id, amount);
                } else {
                    return Err(Box::new(TransactionRecordError{ error_type: TransactionErrorType::NoDepositAmount}))
                }
            }
            TransactionType::Withdrawal => {
                // Handle an withdrawal
                if let Some(amount) = transaction.amount {
                    let new_balance = account_balance.available - amount;
                    if new_balance >= 0.0 {
                        account_balance.available = new_balance;
                    } else {
                        // Insuficient funds, ignore
                        continue;
                    }
                } else {
                    return Err(Box::new(TransactionRecordError{ error_type: TransactionErrorType::NoWithdrawalAmount}))
                }
            }
            TransactionType::Dispute => {
                // Handle a dispute
                // Get the amount from the deposit transaction
                let amount = if let Some(&amount) = deposit_transaction_amounts.get(&transaction.tx_id) {
                    amount
                } else {
                    // Transaction not found, error from the partner
                    continue;
                };
                account_balance.available -= amount;
                account_balance.held += amount;
                disputed_transactions.push(transaction.tx_id);
            }
            TransactionType::Resolve => {
                // Handle a dispute resolution
                // Check if the transaction is disputed
                if !disputed_transactions.contains(&transaction.tx_id) {
                    // Invalid resolution, transaction isn't disputed
                    continue;
                }

                // Get the amount from the deposit transaction
                let amount = if let Some(&amount) = deposit_transaction_amounts.get(&transaction.tx_id) {
                    amount
                } else {
                    // Transaction not found, error from the partner
                    continue;
                };
                account_balance.available += amount;
                account_balance.held -= amount;
            }
            TransactionType::Chargeback => {
                // Handle a chargeback
                // Check if the transaction is disputed
                if !disputed_transactions.contains(&transaction.tx_id) {
                    // Invalid resolution, transaction isn't disputed
                    continue;
                }

                // Get the amount from the deposit transaction
                let amount = if let Some(&amount) = deposit_transaction_amounts.get(&transaction.tx_id) {
                    amount
                } else {
                    // Transaction not found, error from the partner
                    continue;
                };
                account_balance.held -= amount;
                account_balance.locked = true;
            }
        }
    }

    // Generate account balances string
    let mut output = vec![String::from("client, available, held, total, locked")];
    for (_client_id, account_balance) in account_balances {
        output.push(format!("{}", account_balance));
    }

    Ok(output.join("\n"))
}

fn main() {
    // Get CSV path from the command arguments
    
    let csv_file = if let Some(file_path) = env::args().nth(1) {file_path} else {
        println!("No file path in the input arguments");
        // We cannot continue without a CSV file, so we exit with an error code.
        process::exit(1);
    };

    // Process the CSV and abort on uncaught errors
    match process_csv(&Path::new(&csv_file)) {
        Ok(output) => {
            println!("{}", output);
        },
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::process_csv;

    fn test_csv(file_path: &str, expected: &str) {
        let output = process_csv(&Path::new(file_path)).unwrap();
        let expected_output = String::from(expected);
        assert_eq!(output, expected_output);
    }


    #[test]
    fn test_multiple_clients() {
        test_csv("sample_files/multiple_clients.csv", r"client, available, held, total, locked
1, 2.0000, 0.0000, 2.0000, true
2, 0.5000, 0.0000, 0.5000, false
3, 0.0000, 5.5000, 5.5000, false");
    }

    #[test]
    fn test_deposit_withdrawal() {
        test_csv("sample_files/deposit_withdrawal.csv", r"client, available, held, total, locked
1, 1.5000, 0.0000, 1.5000, false
2, 0.5000, 0.0000, 0.5000, false");
    }

    #[test]
    fn test_missing_tx_dispute() {
        let expected = r"client, available, held, total, locked
1, 1.0000, 0.0000, 1.0000, false";
        test_csv("sample_files/dispute_missing_transaction.csv", expected);
    }

    #[test]
    fn test_missing_tx_resolve() {
        let expected = r"client, available, held, total, locked
1, 1.0000, 0.0000, 1.0000, false";
        test_csv("sample_files/resolve_missing_transaction.csv", expected);
    }

    #[test]
    fn test_missing_tx_chargeback() {
        let expected = r"client, available, held, total, locked
1, 1.0000, 0.0000, 1.0000, false";
        test_csv("sample_files/chargeback_missing_transaction.csv", expected);
    }

    #[test]
    fn test_dispute_chargeback() {
        let expected = r"client, available, held, total, locked
1, 0.0000, 0.0000, 0.0000, true";
        test_csv("sample_files/dispute_chargeback.csv", expected);
    }

    #[test]
    fn test_dispute_resolve() {
        let expected = r"client, available, held, total, locked
1, 1.0000, 0.0000, 1.0000, false";
        test_csv("sample_files/dispute_resolve.csv", expected);
    }

    #[test]
    fn test_dispute() {
        let expected = r"client, available, held, total, locked
1, 0.0000, 1.0000, 1.0000, false";
        test_csv("sample_files/dispute.csv", expected);
    }

    #[test]
    fn test_invalid_csv() {
        assert!(process_csv(&Path::new("sample_files/invalid_csv.csv")).is_err());
    }

    #[test]
    fn test_withdrawal_insufficient_funds() {
        let expected = r"client, available, held, total, locked
1, 1.0000, 0.0000, 1.0000, false";
        test_csv("sample_files/withdrawal_insufficient_funds.csv", expected);
    }
    
}
