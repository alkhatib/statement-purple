use crate::types::Transaction;
use std::error::Error;

pub fn process_csv<R>(reader: R) -> Result<(), Box<dyn Error>>
where
    R: std::io::Read,
{
    // Parse CSV
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(reader);

    for result in reader.deserialize::<Transaction>() {
        match result {
            Ok(record) => {
                println!("{:?}", record);
            }
            Err(_) => {
                // do nothing
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_data_only() {
        let csv_data = "type,client,tx,amount";
        let cursor = csv_data.as_bytes();

        let result = process_csv(cursor);
        assert!(result.is_ok());
    }
}
