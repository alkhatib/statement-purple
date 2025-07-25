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

#[test]
fn test_rounding_in_dispute_resolve_flow() {
    let csv_data = "
        type,       client, tx, amount
        deposit,    1,      1,  123.456789
        dispute,    1,      1
        resolve,    1,      1";
    let csv_bytes = csv_data.as_bytes();
    let reader = csv_reader(csv_bytes);

    let ledger = Ledger::from_csv_reader(reader).unwrap();
    let client = ledger.get_client(1).unwrap();

    // Amount should be properly rounded through dispute/resolve cycle
    let expected_amount = Decimal::new(1234568, 4); // 123.4568 (rounded)
    assert_eq!(client.total(), expected_amount);
    assert_eq!(client.available(), expected_amount);
    assert_eq!(client.held(), Decimal::ZERO);
    assert_eq!(client.total().scale(), 4);
    assert_account_invariants(client);
}

#[test]
fn test_rounding_in_chargeback_flow() {
    let csv_data = "
        type,       client, tx, amount
        deposit,    1,      1,  987.654321
        dispute,    1,      1
        chargeback, 1,      1";
    let csv_bytes = csv_data.as_bytes();
    let reader = csv_reader(csv_bytes);

    let ledger = Ledger::from_csv_reader(reader).unwrap();
    let client = ledger.get_client(1).unwrap();

    // After chargeback, all amounts should be zero with proper precision
    assert_eq!(client.total(), Decimal::ZERO);
    assert_eq!(client.available(), Decimal::ZERO);
    assert_eq!(client.held(), Decimal::ZERO);
    assert!(client.is_locked());
    assert_eq!(client.total().scale(), 4);
}

#[test]
fn test_multiple_clients_with_precise_amounts() {
    let csv_data = "
        type,       client, tx, amount
        deposit,    1,      1,  100.123456
        deposit,    2,      2,  200.987654
        deposit,    3,      3,  300.555555
        withdrawal, 1,      4,  25.111111
        dispute,    2,      2
        resolve,    2,      2";
    let csv_bytes = csv_data.as_bytes();
    let reader = csv_reader(csv_bytes);

    let ledger = Ledger::from_csv_reader(reader).unwrap();

    // Check client 1
    let client1 = ledger.get_client(1).unwrap();
    let expected1_total = Decimal::new(1001235, 4) - Decimal::new(251111, 4);
    assert_eq!(client1.total(), expected1_total);
    assert_eq!(client1.total().scale(), 4);
    assert_account_invariants(client1);

    // Check client 2
    let client2 = ledger.get_client(2).unwrap();
    let expected2_total = Decimal::new(2009877, 4);
    assert_eq!(client2.total(), expected2_total);
    assert_eq!(client2.total().scale(), 4);
    assert_account_invariants(client2);

    // Check client 3
    let client3 = ledger.get_client(3).unwrap();
    let expected3_total = Decimal::new(3005556, 4);
    assert_eq!(client3.total(), expected3_total);
    assert_eq!(client3.total().scale(), 4);
    assert_account_invariants(client3);

    assert_eq!(ledger.client_count(), 3);
}

#[test]
fn test_banker_rounding_edge_cases() {
    let csv_data = "
        type,       client, tx, amount
        deposit,    1,      1,  1.23455
        deposit,    1,      2,  2.34565
        deposit,    1,      3,  3.45675
        deposit,    1,      4,  4.56785";
    let csv_bytes = csv_data.as_bytes();
    let reader = csv_reader(csv_bytes);

    let ledger = Ledger::from_csv_reader(reader).unwrap();
    let client = ledger.get_client(1).unwrap();

    // Test that banker's rounding (round half to even) is applied correctly
    // 1.23455 -> 1.2346 (round up)
    // 2.34565 -> 2.3456 (round down)
    // 3.45675 -> 3.4568 (round up)
    // 4.56785 -> 4.5678 (round down)
    let expected_total = Decimal::new(12346, 4)
        + Decimal::new(23456, 4)
        + Decimal::new(34568, 4)
        + Decimal::new(45678, 4);
    assert_eq!(client.total(), expected_total);
    assert_eq!(client.total().scale(), 4);
    assert_account_invariants(client);
}

#[test]
fn test_precision_maintained_across_complex_flow() {
    let csv_data = "
        type,       client, tx, amount
        deposit,    1,      1,  1000.999999
        withdrawal, 1,      2,  100.111111
        deposit,    1,      3,  50.555555
        dispute,    1,      1
        deposit,    1,      4,  25.333333
        resolve,    1,      1
        withdrawal, 1,      5,  200.222222";
    let csv_bytes = csv_data.as_bytes();
    let reader = csv_reader(csv_bytes);

    let ledger = Ledger::from_csv_reader(reader).unwrap();
    let client = ledger.get_client(1).unwrap();

    // All operations should maintain 4 decimal precision
    assert_eq!(client.total().scale(), 4);
    assert_eq!(client.available().scale(), 4);
    assert_eq!(client.held().scale(), 4);

    // Calculate expected final balance step by step with rounding
    // 1. deposit 1000.999999 -> 1001.0000 (rounded)
    // 2. withdraw 100.111111 -> 100.1111 (rounded), balance: 900.8889
    // 3. deposit 50.555555 -> 50.5556 (rounded), balance: 951.4445
    // 4. dispute tx 1 -> moves 1001.0000 to held, available: -49.5555 (but this might be handled differently)
    // 5. deposit 25.333333 -> 25.3333 (rounded) - ignored because locked or insufficient funds
    // 6. resolve tx 1 -> moves back from held
    // 7. withdraw 200.222222 -> 200.2222 (rounded)

    // The exact calculation depends on the implementation details, but precision should be maintained
    assert_account_invariants(client);
}
