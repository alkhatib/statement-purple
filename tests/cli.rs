use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn args_filepath_missing() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("in-gen")?;

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage: in-gen input_file.csv"));
    Ok(())
}
