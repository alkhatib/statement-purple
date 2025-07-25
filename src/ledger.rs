use std::{collections::HashMap, io::Read};

use rust_decimal::Decimal;

use crate::Transaction;

#[derive(Debug)]
struct ClientTransactions {
    disputable: HashMap<u32, Decimal>, // tx_id -> amount
    disputed: HashMap<u32, Decimal>,   // tx_id -> amount
}

impl ClientTransactions {
    fn new() -> Self {
        ClientTransactions {
            disputable: HashMap::new(),
            disputed: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct ClientAccount {
    client_id: u16,
    total: Decimal,
    held: Decimal,
    locked: bool,
    transactions: ClientTransactions,
}

impl ClientAccount {
    fn new(client_id: u16) -> Self {
        ClientAccount {
            client_id,
            total: Decimal::new(0, 4),
            held: Decimal::new(0, 4),
            locked: false,
            transactions: ClientTransactions::new(),
        }
    }

    fn deposit(&mut self, tx_id: u32, amount: Option<Decimal>) {
        match amount {
            None => (),
            Some(amount) => {
                self.total += amount;
                self.transactions.disputable.insert(tx_id, amount);
            }
        }
    }
}

type ClientMap = HashMap<u16, ClientAccount>;

pub fn process_transactions<R: Read>(mut reader: csv::Reader<R>) -> crate::Result<ClientMap> {
    let mut client_map = ClientMap::new();

    for result in reader.deserialize::<Transaction>() {
        match result {
            Ok(transaction) => {
                println!("{transaction:?}");
                match transaction.transaction_type {
                    crate::TransactionType::Deposit => {
                        //check the hashmap
                        let client_account = client_map
                            .entry(transaction.client)
                            .or_insert(ClientAccount::new(transaction.client));
                        client_account.deposit(transaction.tx_id, transaction.amount);
                    }
                    crate::TransactionType::Withdrawal => (),
                    crate::TransactionType::Dispute => (),
                    crate::TransactionType::Resolve => (),
                    crate::TransactionType::Chargeback => (),
                }
            }
            Err(_) => {
                println!("error")
            }
        }
    }
    Ok(client_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal::Decimal;

    #[test]
    fn test_add_deposit_transaction_to_account() {
        let csv_data = "type,client,tx,amount
            deposit,1,1,1";
        let csv_bytes = csv_data.as_bytes();

        let reader = crate::csv_reader(csv_bytes);

        let result = process_transactions(reader).unwrap();
        assert_eq!(result.get(&1u16).unwrap().total, Decimal::new(1, 0));
    }
}
