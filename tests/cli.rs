use assert_cmd::Command;
use predicates::prelude::*;
use std::process::Command as StdCommand;
use std::fs;

#[test]
fn fastq_counts_match_expected() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("rusty"));
    let dir = env!("CARGO_MANIFEST_DIR");
    let p1 = format!("{}/tests/sample_R1.fastq", dir);
    let p2 = format!("{}/tests/sample_R2.fastq", dir);

    // Regenerate sample FASTQ files for a reproducible integration test run.
    let gen = format!("{}/tests/generate_test_fastq.py", dir);
    // generate R1
    let out1 = StdCommand::new("python3")
        .arg(&gen)
        .arg("--read_length")
        .arg("100")
        .arg("--number")
        .arg("20")
        .output()?;
    if !out1.status.success() {
        panic!("generator failed: {}", String::from_utf8_lossy(&out1.stderr));
    }
    fs::write(&p1, &out1.stdout)?;

    // generate R2
    let out2 = StdCommand::new("python3")
        .arg(&gen)
        .arg("--read_length")
        .arg("100")
        .arg("--number")
        .arg("20")
        .output()?;
    if !out2.status.success() {
        panic!("generator failed: {}", String::from_utf8_lossy(&out2.stderr));
    }
    fs::write(&p2, &out2.stdout)?;
    cmd.args(["--p1", &p1, "--p2", &p2]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("reads_R1: 20"))
        .stdout(predicate::str::contains("reads_R2: 20"))
        .stdout(predicate::str::contains("pairs: 20"));
    Ok(())
}
