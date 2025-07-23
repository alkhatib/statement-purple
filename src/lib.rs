pub mod csv_parsing;
pub mod types;

// Re-export commonly used types for convenience
pub use csv_parsing::process_csv;
pub use types::{Transaction, TransactionType};

pub fn get_input_file_path() -> Result<String, String> {
    let arg = std::env::args().nth(1);
    let Some(path) = arg else {
        return Err("Usage: in-gen input_file.csv".to_string());
    };

    // check that file exists
    if !std::path::Path::new(&path).exists() {
        return Err(format!("file {path} does not exist"));
    }

    Ok(path)
}
