use std::{collections::HashMap, fmt::Display, io::Read};

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
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
    #[serde(skip)] // Private do not serialize TODO: Separate Output Struct
    transactions: ClientTransactions,
}

impl Display for ClientAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ClientAccount {{ client_id: {client}, total: {total}, held: {held}, available: {available}, locked: {locked} }}",
            client = self.client,
            total = self.total,
            held = self.held,
            available = self.available,
            locked = self.locked,
        )
    }
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

    pub fn available(&self) -> Decimal {
        self.available
    }

    pub fn total(&self) -> Decimal {
        self.total
    }

    pub fn held(&self) -> Decimal {
        self.held
    }

    pub fn is_locked(&self) -> bool {
        self.locked
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
                    ledger.process_transaction(transaction)?;
                }
                Err(_) => {
                    // This is where we can handle any incorrect CSV data
                    // either by logging or metrics tracking
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

    // Helper functions for test setup
    fn create_test_account(client_id: u16) -> ClientAccount {
        ClientAccount::new(client_id)
    }

    fn create_account_with_balance(client_id: u16, total: i64, held: i64) -> ClientAccount {
        let mut account = ClientAccount::new(client_id);
        account.total = Decimal::new(total, 0);
        account.held = Decimal::new(held, 0);
        account.update_available();
        account
    }

    fn assert_account_invariants(account: &ClientAccount) {
        assert_eq!(account.available, account.total - account.held);
        assert!(account.total >= Decimal::ZERO);
        assert!(account.held >= Decimal::ZERO);
        assert!(account.available >= Decimal::ZERO || account.locked);
    }

    // Unit Tests
    mod unit_tests {
        use super::*;

        #[test]
        fn test_new_account_has_zero_balance() {
            // Arrange & Act
            let account = create_test_account(1);

            // Assert
            assert_eq!(account.client, 1);
            assert_eq!(account.total, Decimal::ZERO);
            assert_eq!(account.held, Decimal::ZERO);
            assert_eq!(account.available, Decimal::ZERO);
            assert!(!account.locked);
            assert_account_invariants(&account);
        }

        #[test]
        fn test_deposit_increases_total_and_available() {
            // Arrange
            let mut account = create_test_account(1);
            let deposit_amount = Decimal::new(100, 0);

            // Act
            let result = account.deposit(1, deposit_amount);

            // Assert
            assert!(result.is_ok());
            assert_eq!(account.total, deposit_amount);
            assert_eq!(account.available, deposit_amount);
            assert_eq!(account.held, Decimal::ZERO);
            assert_account_invariants(&account);
        }

        #[test]
        fn test_deposit_zero_or_negative_amount_ignored() {
            // Arrange
            let mut account = create_test_account(1);
            let initial_total = account.total;

            // Act & Assert - zero amount
            assert!(account.deposit(1, Decimal::ZERO).is_ok());
            assert_eq!(account.total, initial_total);

            // Act & Assert - negative amount
            assert!(account.deposit(2, Decimal::new(-50, 0)).is_ok());
            assert_eq!(account.total, initial_total);
            assert_account_invariants(&account);
        }

        #[test]
        fn test_duplicate_deposit_transaction_ignored() {
            // Arrange
            let mut account = create_test_account(1);
            let amount = Decimal::new(100, 0);

            // Act - first deposit
            assert!(account.deposit(1, amount).is_ok());
            let balance_after_first = account.total;

            // Act - duplicate transaction
            assert!(account.deposit(1, amount).is_ok());

            // Assert
            assert_eq!(account.total, balance_after_first);
            assert_account_invariants(&account);
        }

        #[test]
        fn test_withdrawal_decreases_balance() {
            // Arrange
            let mut account = create_account_with_balance(1, 100, 0);

            // Act
            let result = account.withdraw(1, Decimal::new(30, 0));

            // Assert
            assert!(result.is_ok());
            assert_eq!(account.total, Decimal::new(70, 0));
            assert_eq!(account.available, Decimal::new(70, 0));
            assert_account_invariants(&account);
        }

        #[test]
        fn test_withdrawal_insufficient_funds_ignored() {
            // Arrange
            let mut account = create_account_with_balance(1, 50, 0);
            let initial_total = account.total;

            // Act - try to withdraw more than available
            let result = account.withdraw(1, Decimal::new(100, 0));

            // Assert
            assert!(result.is_ok());
            assert_eq!(account.total, initial_total); // Balance unchanged
            assert_account_invariants(&account);
        }

        #[test]
        fn test_dispute_moves_funds_to_held() {
            // Arrange
            let mut account = create_test_account(1);
            let amount = Decimal::new(100, 0);
            account.deposit(1, amount).unwrap();

            // Act
            let result = account.dispute(1);

            // Assert
            assert!(result.is_ok());
            assert_eq!(account.total, amount);
            assert_eq!(account.held, amount);
            assert_eq!(account.available, Decimal::ZERO);
            assert_account_invariants(&account);
        }

        #[test]
        fn test_dispute_nonexistent_transaction_ignored() {
            // Arrange
            let mut account = create_account_with_balance(1, 100, 0);
            let initial_state = (account.total, account.held, account.available);

            // Act
            let result = account.dispute(999); // Non-existent transaction

            // Assert
            assert!(result.is_ok());
            assert_eq!(
                (account.total, account.held, account.available),
                initial_state
            );
            assert_account_invariants(&account);
        }

        #[test]
        fn test_resolve_dispute_releases_held_funds() {
            // Arrange
            let mut account = create_test_account(1);
            let amount = Decimal::new(100, 0);
            account.deposit(1, amount).unwrap();
            account.dispute(1).unwrap();

            // Act
            let result = account.resolve(1);

            // Assert
            assert!(result.is_ok());
            assert_eq!(account.total, amount);
            assert_eq!(account.held, Decimal::ZERO);
            assert_eq!(account.available, amount);
            assert_account_invariants(&account);
        }

        #[test]
        fn test_chargeback_locks_account_and_removes_funds() {
            // Arrange
            let mut account = create_test_account(1);
            let amount = Decimal::new(100, 0);
            account.deposit(1, amount).unwrap();
            account.dispute(1).unwrap();

            // Act
            let result = account.chargeback(1);

            // Assert
            assert!(result.is_ok());
            assert!(account.locked);
            assert_eq!(account.total, Decimal::ZERO);
            assert_eq!(account.held, Decimal::ZERO);
            assert_eq!(account.available, Decimal::ZERO);
        }

        #[test]
        fn test_locked_account_ignores_all_operations() {
            // Arrange
            let mut account = create_test_account(1);
            account.locked = true;

            // Act & Assert - all operations should be ignored
            assert!(account.deposit(1, Decimal::new(100, 0)).is_ok());
            assert_eq!(account.total, Decimal::ZERO);

            assert!(account.withdraw(2, Decimal::new(50, 0)).is_ok());
            assert_eq!(account.total, Decimal::ZERO);

            assert!(account.dispute(3).is_ok());
            assert_eq!(account.held, Decimal::ZERO);

            assert_account_invariants(&account);
        }

        #[test]
        fn test_display_implementation() {
            // Arrange
            let mut account = create_test_account(123);
            account.deposit(1, Decimal::new(100, 0)).unwrap();
            account.dispute(1).unwrap();

            // Act
            let display_string = format!("{}", account);

            // Assert
            assert!(display_string.contains("client_id: 123"));
            assert!(display_string.contains("total: 100"));
            assert!(display_string.contains("held: 100"));
            assert!(display_string.contains("available: 0"));
            assert!(display_string.contains("locked: false"));
        }

        #[test]
        fn test_withdrawal_zero_or_negative_amount_ignored() {
            // Arrange
            let mut account = create_account_with_balance(1, 100, 0);
            let initial_total = account.total;

            // Act & Assert - zero amount
            assert!(account.withdraw(1, Decimal::ZERO).is_ok());
            assert_eq!(account.total, initial_total);

            // Act & Assert - negative amount
            assert!(account.withdraw(2, Decimal::new(-50, 0)).is_ok());
            assert_eq!(account.total, initial_total);

            assert_account_invariants(&account);
        }

        #[test]
        fn test_dispute_already_disputed_transaction_ignored() {
            // Arrange
            let mut account = create_test_account(1);
            account.deposit(1, Decimal::new(100, 0)).unwrap();
            account.dispute(1).unwrap();
            let state_after_first_dispute = (account.total, account.held, account.available);

            // Act - try to dispute again
            assert!(account.dispute(1).is_ok());

            // Assert - state should be unchanged
            assert_eq!(
                (account.total, account.held, account.available),
                state_after_first_dispute
            );
            assert_account_invariants(&account);
        }

        #[test]
        fn test_resolve_non_disputed_transaction_ignored() {
            // Arrange
            let mut account = create_account_with_balance(1, 100, 50);
            let initial_state = (account.total, account.held, account.available);

            // Act
            assert!(account.resolve(999).is_ok()); // Non-existent transaction

            // Assert
            assert_eq!(
                (account.total, account.held, account.available),
                initial_state
            );
            assert_account_invariants(&account);
        }

        #[test]
        fn test_chargeback_non_disputed_transaction_ignored() {
            // Arrange
            let mut account = create_account_with_balance(1, 100, 0);
            let initial_state = (
                account.total,
                account.held,
                account.available,
                account.locked,
            );

            // Act
            assert!(account.chargeback(999).is_ok()); // Non-existent transaction

            // Assert
            assert_eq!(
                (
                    account.total,
                    account.held,
                    account.available,
                    account.locked
                ),
                initial_state
            );
            assert_account_invariants(&account);
        }

        #[test]
        fn test_deposit_to_disputed_transaction_id_ignored() {
            // Arrange
            let mut account = create_test_account(1);
            account.deposit(1, Decimal::new(100, 0)).unwrap();
            account.dispute(1).unwrap();
            let balance_before = account.total;

            // Act - try to deposit to disputed transaction ID
            assert!(account.deposit(1, Decimal::new(50, 0)).is_ok());

            // Assert - should be ignored
            assert_eq!(account.total, balance_before);
            assert_account_invariants(&account);
        }

        #[test]
        fn test_ledger_new_creates_empty_ledger() {
            // Act
            let ledger = Ledger::new();

            // Assert
            assert_eq!(ledger.client_count(), 0);
        }

        #[test]
        fn test_ledger_default_creates_empty_ledger() {
            // Act
            let ledger = Ledger::default();

            // Assert
            assert_eq!(ledger.client_count(), 0);
        }

        #[test]
        fn test_get_client_nonexistent_returns_none() {
            // Arrange
            let ledger = Ledger::new();

            // Act & Assert
            assert!(ledger.get_client(999).is_none());
        }

        #[test]
        fn test_get_client_balance_nonexistent_returns_none() {
            // Arrange
            let ledger = Ledger::new();

            // Act & Assert
            assert!(ledger.get_client_balance(999).is_none());
        }

        #[test]
        fn test_get_client_balance_returns_correct_values() {
            // Arrange
            let mut ledger = Ledger::new();
            let transaction = crate::Transaction {
                transaction_type: crate::TransactionType::Deposit,
                client: 1,
                tx_id: 1,
                amount: Some(Decimal::new(100, 0)),
            };
            ledger.process_transaction(transaction).unwrap();

            // Act
            let balance = ledger.get_client_balance(1);

            // Assert
            assert!(balance.is_some());
            let (total, held, available) = balance.unwrap();
            assert_eq!(total, Decimal::new(100, 0));
            assert_eq!(held, Decimal::ZERO);
            assert_eq!(available, Decimal::new(100, 0));
        }

        #[test]
        fn test_iter_clients() {
            // Arrange
            let mut ledger = Ledger::new();
            let transaction1 = crate::Transaction {
                transaction_type: crate::TransactionType::Deposit,
                client: 1,
                tx_id: 1,
                amount: Some(Decimal::new(100, 0)),
            };
            let transaction2 = crate::Transaction {
                transaction_type: crate::TransactionType::Deposit,
                client: 2,
                tx_id: 2,
                amount: Some(Decimal::new(200, 0)),
            };
            ledger.process_transaction(transaction1).unwrap();
            ledger.process_transaction(transaction2).unwrap();

            // Act
            let clients: Vec<_> = ledger.iter_clients().collect();

            // Assert
            assert_eq!(clients.len(), 2);
            assert_eq!(ledger.client_count(), 2);
        }

        #[test]
        fn test_process_transaction_locked_client_ignored() {
            // Arrange
            let mut ledger = Ledger::new();

            // First deposit and chargeback to lock the account
            let deposit = crate::Transaction {
                transaction_type: crate::TransactionType::Deposit,
                client: 1,
                tx_id: 1,
                amount: Some(Decimal::new(100, 0)),
            };
            let dispute = crate::Transaction {
                transaction_type: crate::TransactionType::Dispute,
                client: 1,
                tx_id: 1,
                amount: None,
            };
            let chargeback = crate::Transaction {
                transaction_type: crate::TransactionType::Chargeback,
                client: 1,
                tx_id: 1,
                amount: None,
            };

            ledger.process_transaction(deposit).unwrap();
            ledger.process_transaction(dispute).unwrap();
            ledger.process_transaction(chargeback).unwrap();

            let client = ledger.get_client(1).unwrap();
            assert!(client.is_locked());
            let balance_before = client.total();

            // Try to process another transaction on locked account
            let another_deposit = crate::Transaction {
                transaction_type: crate::TransactionType::Deposit,
                client: 1,
                tx_id: 2,
                amount: Some(Decimal::new(50, 0)),
            };

            // Act
            ledger.process_transaction(another_deposit).unwrap();

            // Assert - balance should be unchanged
            let client_after = ledger.get_client(1).unwrap();
            assert_eq!(client_after.total(), balance_before);
            assert!(client_after.is_locked());
        }

        #[test]
        fn test_from_csv_reader_with_malformed_data() {
            // Arrange
            let csv_data = "
                type,client,tx,amount
                deposit,1,1,100
                invalid_row_with_missing_data
                withdrawal,1,2,50";
            let csv_bytes = csv_data.as_bytes();
            let reader = crate::csv_reader(csv_bytes);

            // Act
            let result = Ledger::from_csv_reader(reader);

            // Assert - should succeed and skip malformed rows
            assert!(result.is_ok());
            let ledger = result.unwrap();
            let client = ledger.get_client(1).unwrap();
            assert_eq!(client.total(), Decimal::new(50, 0)); // 100 - 50
        }
    }

    // Property-Based Testing
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
        #[test]
        fn test_account_invariants_always_hold(
            deposits in prop::collection::vec(1u32..1000, 0..10),
            amounts in prop::collection::vec(1i64..10000, 0..10)
        ) {
            let mut account = create_test_account(1);

            // Apply random deposits
            for (i, &amount) in amounts.iter().enumerate() {
                if i < deposits.len() {
                    let _ = account.deposit(deposits[i], Decimal::new(amount, 0));
                }
            }

            // Invariants should always hold
            assert_account_invariants(&account);
            prop_assert!(account.total >= Decimal::ZERO);
            prop_assert!(account.held >= Decimal::ZERO);
            prop_assert!(account.available == account.total - account.held);
        }

        #[test]
        fn test_deposit_then_dispute_maintains_total(
            tx_id in 1u32..1000,
            amount in 1i64..10000
        ) {
            let mut account = create_test_account(1);
            let deposit_amount = Decimal::new(amount, 0);

            // Deposit then dispute
            account.deposit(tx_id, deposit_amount).unwrap();
            let total_before_dispute = account.total;
            account.dispute(tx_id).unwrap();

            // Total should remain the same, just moved to held
            prop_assert_eq!(account.total, total_before_dispute);
            prop_assert_eq!(account.held, deposit_amount);
            prop_assert_eq!(account.available, Decimal::ZERO);
            assert_account_invariants(&account);
        }

        #[test]
        fn test_withdrawal_never_exceeds_available(
            initial_amount in 1i64..10000,
            withdrawal_amount in 1i64..20000
        ) {
            let mut account = create_account_with_balance(1, initial_amount, 0);
            let initial_available = account.available;

            account.withdraw(1, Decimal::new(withdrawal_amount, 0)).unwrap();

            // Available funds should never go negative (unless locked)
            if !account.locked {
                prop_assert!(account.available >= Decimal::ZERO);
            }

            // If withdrawal was larger than available, balance shouldn't change
            if Decimal::new(withdrawal_amount, 0) > initial_available {
                prop_assert_eq!(account.total, Decimal::new(initial_amount, 0));
            }

            assert_account_invariants(&account);
        }
             }
    }
}
