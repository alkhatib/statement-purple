use std::process;

fn get_input_file_path() -> Result<String, String> {
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

fn main() {
    let path = match get_input_file_path() {
        Ok(path) => path,
        Err(error_msg) => {
            eprintln!("{error_msg}");
            process::exit(1);
        }
    };
}
