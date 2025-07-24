use std::io::Read;

use crate::Transaction;

pub fn process_transactions<R: Read>(mut reader: csv::Reader<R>) {
    for result in reader.deserialize::<Transaction>() {
        match result {
            Ok(transaction) => {
                println!("{transaction:?}")
            }
            Err(_) => {
                println!("error")
            }
        }
    }
}
