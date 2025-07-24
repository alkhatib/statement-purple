use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::error::Error;
use std::io::Write;
use std::process::Command;

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

#[test]
fn csv_extra_whitespace() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("in-gen")?;
    // create a named temp file
    let mut tmp_file = tempfile::NamedTempFile::new()?;

    // write some line with extra whitespace
    writeln!(tmp_file, " type , client , tx  ,  amount  ")?;
    writeln!(tmp_file, "  deposit  ,  1  ,  1  ,  1.00001  ")?;

    cmd.arg(tmp_file.path());
    // assert that the command succeeds
    cmd.assert().success();

    Ok(())
}
