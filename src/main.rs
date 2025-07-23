use std::process;

fn main() {
    let arg = std::env::args().nth(1);
    let Some(_path) = arg else {
        eprintln!("Usage: in-gen input_file.csv");
        process::exit(1);
    };
}
