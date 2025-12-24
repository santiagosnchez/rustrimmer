use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::io::BufWriter;
use flate2::write::GzEncoder;
use flate2::Compression;
use bio::io::fastq;

mod io_utils;
mod trim;

use crate::io_utils::open_input;
use crate::trim::trim_record;

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

    /// Trim low-quality ends (enable trimming mode)
    #[arg(long)]
    trim: bool,

    /// Quality threshold (Phred) for trimming ends; default 20
    #[arg(long, default_value_t = 20)]
    qual: u8,

    /// Minimum length to keep a read after trimming; default 30
    #[arg(long, default_value_t = 30)]
    min_len: usize,

    /// Sliding window size for trimming; use 1 to check single-base quality (default)
    #[arg(long, default_value_t = 1)]
    window: usize,

    /// Output FASTQ file (defaults to stdout). Use .gz to write gzipped output.
    #[arg(long)]
    out: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match (args.input, args.p1, args.p2) {
        (Some(path), None, None) => {
            // single-end mode
            if args.trim {
                // trimming mode
                let reader = open_input(&path)?;
                let fq = fastq::Reader::new(BufReader::new(reader));

                // prepare output writer
                let writer: Box<dyn Write> = match &args.out {
                    Some(o) if o.ends_with(".gz") => {
                        let f = File::create(o)?;
                        Box::new(GzEncoder::new(BufWriter::new(f), Compression::default()))
                    }
                    Some(o) => {
                        let f = File::create(o)?;
                        Box::new(BufWriter::new(f))
                    }
                    None => Box::new(io::stdout()),
                };
                let mut fqw = fastq::Writer::new(writer);

                let mut kept: u64 = 0;
                let mut dropped: u64 = 0;

                for result in fq.records() {
                    let rec = result?;
                    if let Some((seq, qual)) = trim_record(rec.qual(), rec.seq(), args.qual, args.min_len, args.window) {
                        // write record with same id/desc
                        fqw.write(&rec.id(), rec.desc(), &seq, &qual)?;
                        kept += 1;
                    } else {
                        dropped += 1;
                    }
                }

                eprintln!("trimmed kept: {}  dropped: {}", kept, dropped);
            } else {
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
