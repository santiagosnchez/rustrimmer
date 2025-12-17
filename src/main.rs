use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use bio::io::fastq;
use flate2::read::MultiGzDecoder;

#[derive(Parser)]
#[command(author, version, about = "Simple FASTQ reader: counts reads and bases")]
struct Args {
    /// Input FASTQ (use '-' for stdin). Supports .gz compressed files.
    /// Provide either a single input or both `--p1` and `--p2` for paired-end files.
    input: Option<String>,

    /// Paired-end R1 (e.g. sample_R1.fastq or .fastq.gz)
    #[arg(long)]
    p1: Option<String>,

    /// Paired-end R2 (e.g. sample_R2.fastq or .fastq.gz)
    #[arg(long)]
    p2: Option<String>,
}

fn open_input(path: &str) -> Result<Box<dyn Read>, Box<dyn Error>> {
    if path == "-" {
        // Peek stdin to detect gzip magic (0x1f 0x8b)
        let mut br = BufReader::new(io::stdin());
        let buf = br.fill_buf()?;
        let is_gz = buf.len() >= 2 && buf[0] == 0x1f && buf[1] == 0x8b;
        // end borrow scope
        let _ = buf;
        if is_gz {
            Ok(Box::new(MultiGzDecoder::new(br)))
        } else {
            Ok(Box::new(br))
        }
    } else {
        let f = File::open(path)?;
        let mut br = BufReader::new(f);
        let buf = br.fill_buf()?;
        let is_gz = buf.len() >= 2 && buf[0] == 0x1f && buf[1] == 0x8b;
        let _ = buf;
        if is_gz {
            Ok(Box::new(MultiGzDecoder::new(br)))
        } else {
            Ok(Box::new(br))
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match (args.input, args.p1, args.p2) {
        (Some(path), None, None) => {
            // single-end mode
            let reader = open_input(&path)?;
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
        }
        (None, Some(p1), Some(p2)) => {
            // paired-end mode: process both files and report per-file counts
            let r1 = open_input(&p1)?;
            let r2 = open_input(&p2)?;

            let fq1 = fastq::Reader::new(BufReader::new(r1));
            let fq2 = fastq::Reader::new(BufReader::new(r2));

            let mut reads1: u64 = 0;
            let mut bases1: u64 = 0;
            for result in fq1.records() {
                let rec = result?;
                reads1 += 1;
                bases1 += rec.seq().len() as u64;
            }

            let mut reads2: u64 = 0;
            let mut bases2: u64 = 0;
            for result in fq2.records() {
                let rec = result?;
                reads2 += 1;
                bases2 += rec.seq().len() as u64;
            }

            println!("reads_R1: {}", reads1);
            println!("bases_R1: {}", bases1);
            println!("reads_R2: {}", reads2);
            println!("bases_R2: {}", bases2);

            if reads1 != reads2 {
                eprintln!("warning: R1 and R2 have different read counts ({} != {})", reads1, reads2);
            } else {
                println!("pairs: {}", reads1);
            }
        }
        _ => {
            eprintln!("Error: provide either a positional input or both --p1 and --p2");
            std::process::exit(2);
        }
    }

    Ok(())
}
