use std::io::Read;

pub fn csv_reader<R: Read>(reader: R) -> csv::Reader<R> {
    // Parse CSV
    csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader)
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use rust_decimal::dec;

    use crate::{Transaction, TransactionType};

    use super::*;

    #[test]
    fn header_data_only() -> Result<(), Box<dyn Error>> {
        let csv_data = "type,client,tx,amount";
        let csv_bytes = csv_data.as_bytes();

        let mut reader = csv_reader(csv_bytes);

        let transactions: Vec<Transaction> = reader.deserialize().collect::<Result<Vec<_>, _>>()?;

        assert_eq!(transactions.len(), 0);
        Ok(())
    }

    #[test]
    fn mismatched_length() -> Result<(), Box<dyn Error>> {
        let csv_data = "type,client,tx,amount
            deposit,1,1,1
            deposit,2
            deposit,3,3,3
";
        let csv_bytes = csv_data.as_bytes();

        let mut reader = csv_reader(csv_bytes);

        let transactions: Vec<Result<Transaction, csv::Error>> =
            reader.deserialize::<Transaction>().collect();
        assert!(transactions[0].is_ok());

        // Missing required fields
        assert!(transactions[1].is_err());

        assert!(transactions[2].is_ok());

        if let Ok(ref third) = transactions[2] {
            assert_eq!(third.transaction_type, TransactionType::Deposit);
            assert_eq!(third.client, 3);
            assert_eq!(third.tx_id, 3);
            assert_eq!(third.amount, Some(dec!(3)));
        };

        assert_eq!(transactions.len(), 3);

        Ok(())
    }

    #[test]
    fn parse_ok_missing_optional_amount() -> Result<(), Box<dyn Error>> {
        let csv_data = "type,client,tx,amount
            deposit,1,1";
        let csv_bytes = csv_data.as_bytes();

        let mut reader = csv_reader(csv_bytes);

        let transactions: Vec<Result<Transaction, csv::Error>> =
            reader.deserialize::<Transaction>().collect();

        assert!(transactions[0].is_ok());

        if let Ok(ref first) = transactions[0] {
            assert_eq!(first.transaction_type, TransactionType::Deposit);
            assert_eq!(first.client, 1);
            assert_eq!(first.tx_id, 1);
            assert_eq!(first.amount, None);
        };
        assert_eq!(transactions.len(), 1);

        Ok(())
    }
}
