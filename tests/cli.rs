use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::tempdir;

#[test]
fn fastq_counts_match_expected() -> Result<(), Box<dyn std::error::Error>> {
    let td = tempdir()?; // auto-deleted when td is dropped
    let p1 = td.path().join("sample_R1.fastq");
    let p2 = td.path().join("sample_R2.fastq");

    // generate into temp files
    let gen = format!(
        "{}/tests/generate_test_fastq.py",
        env!("CARGO_MANIFEST_DIR")
    );
    let out1 = StdCommand::new("python3")
        .arg(&gen)
        .arg("--read_length")
        .arg("100")
        .arg("--number")
        .arg("20")
        .output()?;
    fs::write(&p1, &out1.stdout)?;

    let out2 = StdCommand::new("python3")
        .arg(&gen)
        .arg("--read_length")
        .arg("100")
        .arg("--number")
        .arg("20")
        .output()?;
    fs::write(&p2, &out2.stdout)?;

    // use a temp output base inside same tempdir
    let out_base = td.path().join("out");
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("rustrimmer"));
    cmd.args([
        "--p1",
        p1.to_str().unwrap(),
        "--p2",
        p2.to_str().unwrap(),
        "--output",
        out_base.to_str().unwrap(),
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("R1 reads: 20"))
        .stdout(predicate::str::contains("R2 reads: 20"))
        .stdout(predicate::str::contains("total pairs: 20"));

    // optional: assert files exist inside tempdir (accept .fastq, .fastq.gz or .fastq.zst)
    let out_r1 = td.path().join("out_R1.fastq");
    let out_r1_gz = td.path().join("out_R1.fastq.gz");
    let out_r1_zst = td.path().join("out_R1.fastq.zst");
    assert!(out_r1.exists() || out_r1_gz.exists() || out_r1_zst.exists());

    // when test returns, `td` is dropped and the directory+files are removed
    Ok(())
}
