use in_gen::{CsvFileStream, Ledger, error::Result, get_input_file_path};
use std::process;

#[tokio::main]
async fn main() -> Result<()> {
    let path = match get_input_file_path() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    let stream = CsvFileStream::from_path(&path).await?;
    let ledger = Ledger::from_stream(stream).await?;

    // Write client accounts to CSV output
    let mut writer = csv::Writer::from_writer(std::io::stdout());
    for client_account in ledger.iter_clients() {
        writer.serialize(client_account)?;
    }
    writer.flush()?;

    Ok(())
}
