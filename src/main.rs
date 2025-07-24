use in_gen::{csv_reader, error::Result, get_input_file_path, ledger};
use std::fs::File;
use std::process;

fn main() -> Result<()> {
    let path = match get_input_file_path() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    let file = File::open(path)?;

    let transaction_reader = csv_reader(file);

    ledger::process_transactions(transaction_reader);

    Ok(())
}
