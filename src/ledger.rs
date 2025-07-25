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
        if let Some(amount) = amount {
            self.total += amount;
            self.transactions.disputable.insert(tx_id, amount);
        }
    }
}

type ClientMap = HashMap<u16, ClientAccount>;

pub struct Ledger {
    clients: ClientMap,
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            clients: ClientMap::new(),
        }
    }

    pub fn from_csv_reader<R: Read>(mut reader: csv::Reader<R>) -> crate::Result<Self> {
        let mut ledger = Ledger::new();

        for result in reader.deserialize::<Transaction>() {
            match result {
                Ok(transaction) => {
                    println!("{transaction:?}");
                    ledger.process_transaction(transaction)?;
                }
                Err(_) => {
                    println!("error")
                }
            }
        }
        Ok(ledger)
    }

    fn process_transaction(&mut self, transaction: Transaction) -> crate::Result<()> {
        match transaction.transaction_type {
            crate::TransactionType::Deposit => {
                self.process_deposit(transaction.client, transaction.tx_id, transaction.amount)
            }
            crate::TransactionType::Withdrawal => {
                self.process_withdrawal(transaction.client, transaction.tx_id, transaction.amount)
            }
            crate::TransactionType::Dispute => {
                self.process_dispute(transaction.client, transaction.tx_id)
            }
            crate::TransactionType::Resolve => {
                self.process_resolve(transaction.client, transaction.tx_id)
            }
            crate::TransactionType::Chargeback => {
                self.process_chargeback(transaction.client, transaction.tx_id)
            }
        }
    }

    fn process_deposit(
        &mut self,
        client_id: u16,
        tx_id: u32,
        amount: Option<Decimal>,
    ) -> crate::Result<()> {
        let client_account = self
            .clients
            .entry(client_id)
            .or_insert(ClientAccount::new(client_id));
        client_account.deposit(tx_id, amount);
        Ok(())
    }

    fn process_withdrawal(
        &mut self,
        client_id: u16,
        tx_id: u32,
        amount: Option<Decimal>,
    ) -> crate::Result<()> {
        if let Some(client_account) = self.clients.get_mut(&client_id) {
            // TODO: Implement withdrawal logic
        }
        Ok(())
    }

    fn process_dispute(&mut self, client_id: u16, tx_id: u32) -> crate::Result<()> {
        if let Some(client_account) = self.clients.get_mut(&client_id) {
            // TODO: Implement dispute logic
        }
        Ok(())
    }

    fn process_resolve(&mut self, client_id: u16, tx_id: u32) -> crate::Result<()> {
        if let Some(client_account) = self.clients.get_mut(&client_id) {
            // TODO: Implement resolve logic
        }
        Ok(())
    }

    fn process_chargeback(&mut self, client_id: u16, tx_id: u32) -> crate::Result<()> {
        if let Some(client_account) = self.clients.get_mut(&client_id) {
            // TODO: Implement chargeback logic
        }
        Ok(())
    }

    pub fn get_client(&self, client_id: u16) -> Option<&ClientAccount> {
        self.clients.get(&client_id)
    }

    pub fn get_client_balance(&self, client_id: u16) -> Option<(Decimal, Decimal)> {
        self.clients
            .get(&client_id)
            .map(|client| (client.total, client.held))
    }

    pub fn iter_clients(&self) -> impl Iterator<Item = (&u16, &ClientAccount)> {
        self.clients.iter()
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }
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

        let ledger = Ledger::from_csv_reader(reader).unwrap();
        assert_eq!(ledger.get_client(1).unwrap().total, Decimal::new(1, 0));
    }
}
