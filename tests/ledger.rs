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
