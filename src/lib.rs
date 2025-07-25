pub mod csv_parsing;
pub mod error;
pub mod ledger;
pub mod types;

// Re-export commonly used types for convenience
pub use crate::error::{AppError, Result};
pub use csv_parsing::csv_reader;
pub use ledger::Ledger;
pub use types::{Transaction, TransactionType};

pub fn get_input_file_path() -> Result<String> {
    let arg = std::env::args().nth(1);
    let Some(path) = arg else {
        return Err(AppError::MissingArgument(
            "Usage: in-gen input_file.csv".to_string(),
        ));
    };

    // check that file exists
    if !std::path::Path::new(&path).exists() {
        return Err(AppError::FileNotFound(path));
    }

    Ok(path)
}
