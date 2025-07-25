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

#[derive(Debug, serde::Serialize)]
pub struct ClientAccount {
    client: u16,
    total: Decimal,
    held: Decimal,
    available: Decimal,
    locked: bool,
    #[serde(skip)] // Private do not serialize TODO: Separate Output Struct
    transactions: ClientTransactions,
}

impl ClientAccount {
    fn new(client: u16) -> Self {
        ClientAccount {
            client,
            total: Decimal::new(0, 4),
            held: Decimal::new(0, 4),
            available: Decimal::new(0, 4),
            locked: false,
            transactions: ClientTransactions::new(),
        }
    }

    fn update_available(&mut self) {
        self.available = self.total - self.held;
    }

    fn deposit(&mut self, tx_id: u32, amount: Decimal) -> crate::Result<()> {
        if self.locked {
            return Ok(());
        }

        if amount <= Decimal::ZERO {
            return Ok(());
        }

        if self.transactions.disputable.contains_key(&tx_id) {
            return Ok(()); // duplicate transaction id
        }

        if self.transactions.disputed.contains_key(&tx_id) {
            return Ok(()); // duplicate transaction id
        }

        self.total += amount;
        self.update_available();
        self.transactions.disputable.insert(tx_id, amount);

        Ok(())
    }

    fn withdraw(&mut self, _tx_id: u32, amount: Decimal) -> crate::Result<()> {
        if self.locked {
            return Ok(());
        }

        if amount <= Decimal::ZERO {
            return Ok(());
        }

        // Assumption: only available funds are allowed for withdrawal
        if amount <= self.available() {
            self.total -= amount;
            self.update_available();
        }

        Ok(())
    }

    fn dispute(&mut self, tx_id: u32) -> crate::Result<()> {
        if self.locked {
            return Ok(());
        }

        if let Some(amount) = self.transactions.disputable.remove(&tx_id) {
            // TODO: check if the amount is sufficient?
            self.held += amount;
            self.update_available();
            self.transactions.disputed.insert(tx_id, amount);
        }
        Ok(())
    }

    fn resolve(&mut self, tx_id: u32) -> crate::Result<()> {
        if self.locked {
            return Ok(());
        }

        if let Some(amount) = self.transactions.disputed.remove(&tx_id) {
            // TODO: check if the amount is sufficient?
            self.held -= amount;
            self.update_available(); // FIXME: better method to keep this in sync

            // Assumption: Resolved transactions can be disputed again in the future
            self.transactions.disputable.insert(tx_id, amount);
        }
        Ok(())
    }

    fn chargeback(&mut self, tx_id: u32) -> crate::Result<()> {
        if self.locked {
            return Ok(());
        }

        if let Some(amount) = self.transactions.disputed.remove(&tx_id) {
            self.locked = true;
            // TODO: can total or held be less that amount?
            self.total -= amount;
            self.held -= amount;
        }
        Ok(())
    }

    fn available(&self) -> Decimal {
        self.total - self.held
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
        let client = self.get_or_insert(transaction.client);
        if client.locked {
            return Ok(());
        }
        match transaction.transaction_type {
            crate::TransactionType::Deposit => {
                client.deposit(transaction.tx_id, transaction.amount.unwrap_or_default())
            }
            crate::TransactionType::Withdrawal => {
                client.withdraw(transaction.tx_id, transaction.amount.unwrap_or_default())
            }
            crate::TransactionType::Dispute => client.dispute(transaction.tx_id),
            crate::TransactionType::Resolve => client.resolve(transaction.tx_id),
            crate::TransactionType::Chargeback => client.chargeback(transaction.tx_id),
        }
    }

    fn get_or_insert(&mut self, client_id: u16) -> &mut ClientAccount {
        self.clients
            .entry(client_id)
            .or_insert(ClientAccount::new(client_id))
    }

    pub fn get_client(&self, client_id: u16) -> Option<&ClientAccount> {
        self.clients.get(&client_id)
    }

    pub fn get_client_balance(&self, client_id: u16) -> Option<(Decimal, Decimal, Decimal)> {
        self.clients
            .get(&client_id)
            .map(|client| (client.total, client.held, client.available()))
    }

    pub fn iter_clients(&self) -> impl Iterator<Item = &ClientAccount> {
        self.clients.values()
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal::Decimal;

    fn assert_account_invariants(account: &ClientAccount) {
        assert_eq!(account.available, account.total - account.held);
    }

    #[test]
    fn test_add_deposit_transaction_to_account() {
        let csv_data = "type,client,tx,amount
            deposit,1,1,1";
        let csv_bytes = csv_data.as_bytes();

        let reader = crate::csv_reader(csv_bytes);

        let ledger = Ledger::from_csv_reader(reader).unwrap();
        assert_eq!(ledger.get_client(1).unwrap().total, Decimal::new(1, 0));
        assert_account_invariants(&ledger.get_client(1).unwrap());
    }
}
