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

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("in-gen")?;

    let path = "file/does/not/exist";
    cmd.arg(path);
    let expected_error = format!("file {path} does not exist");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(expected_error));
    Ok(())
}
