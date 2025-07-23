use std::process;

fn main() {
    let arg = std::env::args().nth(1);
    let Some(path) = arg else {
        eprintln!("Usage: in-gen input_file.csv");
        process::exit(1);
    };
    // check that file exists
    if !std::path::Path::new(&path).exists() {
        eprintln!("file {path} does not exist");
        process::exit(1);
    }
}
