use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn fastq_counts_match_expected() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rusty")?;
    cmd.arg("tests/sample.fastq");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("reads: 3"))
        .stdout(predicate::str::contains("bases: 45"));
    Ok(())
}
