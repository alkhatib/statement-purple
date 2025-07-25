use in_gen::{Ledger, csv_reader};
use rust_decimal::Decimal;

fn assert_account_invariants(account: &in_gen::ledger::ClientAccount) {
    assert_eq!(account.available(), account.total() - account.held());
    assert!(account.total() >= Decimal::ZERO);
    assert!(account.held() >= Decimal::ZERO);
    assert!(account.available() >= Decimal::ZERO || account.is_locked());
}

#[test]
fn test_complete_transaction_flow() {
    let csv_data = "
        type,   client, tx, amount
        deposit,    1,  1,  100
        deposit,    1,  2,  50
        withdrawal, 1,  3,  25
        dispute,    1,  1
        resolve,    1,  1";
    let csv_bytes = csv_data.as_bytes();
    let reader = csv_reader(csv_bytes);

    let ledger = Ledger::from_csv_reader(reader).unwrap();
    let client = ledger.get_client(1).unwrap();

    assert_eq!(client.total(), Decimal::new(125, 0)); // 100 + 50 - 25
    assert_eq!(client.held(), Decimal::new(0, 0));
    assert_eq!(client.available(), Decimal::new(125, 0));
    assert!(!client.is_locked());
    assert_account_invariants(client);
}

#[test]
fn test_chargeback_scenario() {
    let csv_data = "
        type,       client, tx, amount
        deposit,    1,      1,  100
        dispute,    1,      1
        chargeback, 1,      1";
    let csv_bytes = csv_data.as_bytes();
    let reader = csv_reader(csv_bytes);

    let ledger = Ledger::from_csv_reader(reader).unwrap();
    let client = ledger.get_client(1).unwrap();

    assert_eq!(client.total(), Decimal::new(0, 0));
    assert_eq!(client.held(), Decimal::new(0, 0));
    assert_eq!(client.available(), Decimal::new(0, 0));
    assert!(client.is_locked());
}

#[test]
fn test_multiple_clients_scenario() {
    let csv_data = "
        type,       client, tx, amount
        deposit,    1,      1,  100.0
        deposit,    2,      2,  200.5
        withdrawal, 1,      3,  25.0
        dispute,    2,      2
        resolve,    2,      2";
    let csv_bytes = csv_data.as_bytes();
    let reader = csv_reader(csv_bytes);

    let ledger = Ledger::from_csv_reader(reader).unwrap();
    
    // Check client 1
    let client1 = ledger.get_client(1).unwrap();
    assert_eq!(client1.total(), Decimal::new(75, 0)); // 100 - 25
    assert_eq!(client1.held(), Decimal::new(0, 0));
    assert_eq!(client1.available(), Decimal::new(75, 0));
    assert!(!client1.is_locked());
    assert_account_invariants(client1);

    // Check client 2
    let client2 = ledger.get_client(2).unwrap();
    assert_eq!(client2.total(), Decimal::new(2005, 1)); // 200.5
    assert_eq!(client2.held(), Decimal::new(0, 0));
    assert_eq!(client2.available(), Decimal::new(2005, 1));
    assert!(!client2.is_locked());
    assert_account_invariants(client2);

    // Check ledger state
    assert_eq!(ledger.client_count(), 2);
}

#[test]
fn test_duplicate_transaction_ids_ignored() {
    let csv_data = "
        type,       client, tx, amount
        deposit,    1,      1,  100
        deposit,    1,      1,  50
        withdrawal, 1,      2,  25";
    let csv_bytes = csv_data.as_bytes();
    let reader = csv_reader(csv_bytes);

    let ledger = Ledger::from_csv_reader(reader).unwrap();
    let client = ledger.get_client(1).unwrap();

    // Second deposit should be ignored due to duplicate tx_id
    assert_eq!(client.total(), Decimal::new(75, 0)); // 100 - 25, not 150 - 25
    assert_eq!(client.available(), Decimal::new(75, 0));
    assert_account_invariants(client);
}

#[test]
fn test_transactions_after_chargeback_ignored() {
    let csv_data = "
        type,       client, tx, amount
        deposit,    1,      1,  100
        dispute,    1,      1
        chargeback, 1,      1
        deposit,    1,      2,  50
        withdrawal, 1,      3,  10";
    let csv_bytes = csv_data.as_bytes();
    let reader = csv_reader(csv_bytes);

    let ledger = Ledger::from_csv_reader(reader).unwrap();
    let client = ledger.get_client(1).unwrap();

    // All transactions after chargeback should be ignored
    assert_eq!(client.total(), Decimal::new(0, 0));
    assert_eq!(client.held(), Decimal::new(0, 0));
    assert_eq!(client.available(), Decimal::new(0, 0));
    assert!(client.is_locked());
}

#[test]
fn test_dispute_resolve_dispute_cycle() {
    let csv_data = "
        type,       client, tx, amount
        deposit,    1,      1,  100
        dispute,    1,      1
        resolve,    1,      1
        dispute,    1,      1";
    let csv_bytes = csv_data.as_bytes();
    let reader = csv_reader(csv_bytes);

    let ledger = Ledger::from_csv_reader(reader).unwrap();
    let client = ledger.get_client(1).unwrap();

    // Should end up with disputed transaction again
    assert_eq!(client.total(), Decimal::new(100, 0));
    assert_eq!(client.held(), Decimal::new(100, 0));
    assert_eq!(client.available(), Decimal::new(0, 0));
    assert!(!client.is_locked());
    assert_account_invariants(client);
}

#[test]
fn test_zero_and_negative_amounts_ignored() {
    let csv_data = "
        type,       client, tx, amount
        deposit,    1,      1,  0
        deposit,    1,      2,  -50
        deposit,    1,      3,  100
        withdrawal, 1,      4,  0
        withdrawal, 1,      5,  -25
        withdrawal, 1,      6,  25";
    let csv_bytes = csv_data.as_bytes();
    let reader = csv_reader(csv_bytes);

    let ledger = Ledger::from_csv_reader(reader).unwrap();
    let client = ledger.get_client(1).unwrap();

    // Only valid transactions should be processed
    assert_eq!(client.total(), Decimal::new(75, 0)); // 100 - 25
    assert_eq!(client.available(), Decimal::new(75, 0));
    assert_account_invariants(client);
}

#[test]
fn test_insufficient_funds_withdrawal_ignored() {
    let csv_data = "
        type,       client, tx, amount
        deposit,    1,      1,  50
        withdrawal, 1,      2,  75
        withdrawal, 1,      3,  25";
    let csv_bytes = csv_data.as_bytes();
    let reader = csv_reader(csv_bytes);

    let ledger = Ledger::from_csv_reader(reader).unwrap();
    let client = ledger.get_client(1).unwrap();

    // First withdrawal should be ignored, second should succeed
    assert_eq!(client.total(), Decimal::new(25, 0)); // 50 - 25
    assert_eq!(client.available(), Decimal::new(25, 0));
    assert_account_invariants(client);
}
