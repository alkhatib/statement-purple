use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;
use std::process::Command;
use tempfile;

#[test]
fn csv_file_empty() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("in-gen")?;
    // create a named temp file
    let tmp_file = tempfile::NamedTempFile::new()?;

    cmd.arg(tmp_file.path());
    // assert that the command succeeds
    cmd.assert().success().stdout(predicate::str::is_empty());

    Ok(())
}
