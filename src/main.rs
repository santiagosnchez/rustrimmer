use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, Read};
use bio::io::fastq;
use flate2::read::MultiGzDecoder;

#[derive(Parser)]
#[command(author, version, about = "Simple FASTQ reader: counts reads and bases")]
struct Args {
    /// Input FASTQ (use '-' for stdin). Supports .gz compressed files.
    input: String,
}

fn open_input(path: &str) -> Result<Box<dyn Read>, Box<dyn Error>> {
    if path == "-" {
        Ok(Box::new(io::stdin()))
    } else if path.ends_with(".gz") {
        let f = File::open(path)?;
        Ok(Box::new(MultiGzDecoder::new(f)))
    } else {
        let f = File::open(path)?;
        Ok(Box::new(f))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let reader = open_input(&args.input)?;
    let fq = fastq::Reader::new(BufReader::new(reader));

    let mut read_count: u64 = 0;
    let mut base_count: u64 = 0;

    for result in fq.records() {
        let rec = result?;
        read_count += 1;
        base_count += rec.seq().len() as u64;
    }

    println!("reads: {}", read_count);
    println!("bases: {}", base_count);

    Ok(())
}
