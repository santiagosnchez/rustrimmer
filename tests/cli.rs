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

#[test]
fn paired_missing_output_errors() -> Result<(), Box<dyn std::error::Error>> {
    let td = tempdir()?;
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
        .arg("50")
        .arg("--number")
        .arg("5")
        .output()?;
    fs::write(&p1, &out1.stdout)?;

    let out2 = StdCommand::new("python3")
        .arg(&gen)
        .arg("--read_length")
        .arg("50")
        .arg("--number")
        .arg("5")
        .output()?;
    fs::write(&p2, &out2.stdout)?;

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("rustrimmer"));
    cmd.args(["--p1", p1.to_str().unwrap(), "--p2", p2.to_str().unwrap()]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--output is required"));

    Ok(())
}

#[test]
fn mutually_exclusive_gz_zstd_errors() -> Result<(), Box<dyn std::error::Error>> {
    let td = tempdir()?;
    let p = td.path().join("sample.fastq");
    let gen = format!(
        "{}/tests/generate_test_fastq.py",
        env!("CARGO_MANIFEST_DIR")
    );
    let out = StdCommand::new("python3")
        .arg(&gen)
        .arg("--read_length")
        .arg("50")
        .arg("--number")
        .arg("5")
        .output()?;
    fs::write(&p, &out.stdout)?;

    let out_name = td.path().join("out_single.fastq");
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("rustrimmer"));
    cmd.args([
        p.to_str().unwrap(),
        "--output",
        out_name.to_str().unwrap(),
        "--gz",
        "--zstd",
    ]);

    cmd.assert().failure().stderr(predicate::str::contains(
        "--gz and --zstd are mutually exclusive",
    ));

    Ok(())
}

#[test]
fn gz_and_zstd_outputs_created() -> Result<(), Box<dyn std::error::Error>> {
    let td = tempdir()?; // auto-deleted
    let p1 = td.path().join("sample_R1.fastq");
    let p2 = td.path().join("sample_R2.fastq");

    let gen = format!(
        "{}/tests/generate_test_fastq.py",
        env!("CARGO_MANIFEST_DIR")
    );
    let out1 = StdCommand::new("python3")
        .arg(&gen)
        .arg("--read_length")
        .arg("50")
        .arg("--number")
        .arg("10")
        .output()?;
    fs::write(&p1, &out1.stdout)?;

    let out2 = StdCommand::new("python3")
        .arg(&gen)
        .arg("--read_length")
        .arg("50")
        .arg("--number")
        .arg("10")
        .output()?;
    fs::write(&p2, &out2.stdout)?;

    // gz case
    let out_base = td.path().join("out_gz");
    let mut cmd_gz = Command::new(assert_cmd::cargo::cargo_bin!("rustrimmer"));
    cmd_gz.args([
        "--p1",
        p1.to_str().unwrap(),
        "--p2",
        p2.to_str().unwrap(),
        "--output",
        out_base.to_str().unwrap(),
        "--gz",
    ]);
    cmd_gz.assert().success();
    let gz_r1 = td.path().join("out_gz_R1.fastq.gz");
    let gz_r2 = td.path().join("out_gz_R2.fastq.gz");
    assert!(gz_r1.exists());
    assert!(gz_r2.exists());

    // (zstd behavior is tested via helpers and separate error checks)

    Ok(())
}
