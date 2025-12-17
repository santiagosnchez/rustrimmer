use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn fastq_counts_match_expected() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rusty")?;
    let dir = env!("CARGO_MANIFEST_DIR");
    let p1 = format!("{}/tests/sample_R1.fastq.gz", dir);
    let p2 = format!("{}/tests/sample_R2.fastq.gz", dir);
    cmd.args(["--p1", &p1, "--p2", &p2]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("reads_R1: 100"))
        .stdout(predicate::str::contains("reads_R2: 100"))
        .stdout(predicate::str::contains("pairs: 100"));
    Ok(())
}
