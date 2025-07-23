use in_gen::{get_input_file_path, process_csv};
use std::error::Error;
use std::process;

fn main() -> Result<(), Box<dyn Error>> {
    let path = match get_input_file_path() {
        Ok(path) => path,
        Err(error_msg) => {
            eprintln!("{error_msg}");
            process::exit(1);
        }
    };

    process_csv(&path)?;

    Ok(())
}
