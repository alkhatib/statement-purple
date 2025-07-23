use crate::types::Transaction;
use std::error::Error;

pub fn process_csv(path: &str) -> Result<(), Box<dyn Error>> {
    // Parse CSV
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)?;

    for result in reader.deserialize() {
        let record: Transaction = result?;
        println!("{:?}", record);
    }

    Ok(())
}
