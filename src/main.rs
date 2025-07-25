use in_gen::{Ledger, csv_reader, error::Result, get_input_file_path};
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

    let ledger = Ledger::from_csv_reader(transaction_reader)?;

    // Example: Print client balances
    for (client_id, _) in ledger.iter_clients() {
        if let Some((total, held)) = ledger.get_client_balance(*client_id) {
            println!("Client {client_id}: Total: {total}, Held: {held}");
        }
    }

    Ok(())
}
