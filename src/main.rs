use bio::io::fastq;
use clap::Parser;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::io::{BufReader, Write};
use std::time::Instant;

mod io_utils;
mod trim;

use crate::io_utils::open_input;
use crate::trim::trim_record;

#[derive(Parser)]
#[command(author, version, about = "Simple FASTQ quality trimmer: removes low-quality bases from read ends using sliding window approach", long_about = None)]
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

    /// Quality threshold (Phred) for trimming ends; default 20
    #[arg(long, default_value_t = 20)]
    qual: u8,

    /// Minimum length to keep a read after trimming; default 30
    #[arg(long, default_value_t = 30)]
    min_len: usize,

    /// Sliding window size for trimming; use 1 to check single-base quality (default)
    #[arg(long, default_value_t = 1)]
    window: usize,

    /// Output base name for paired output files (required for paired mode).
    /// For paired mode this will create `<output>_R1.fastq(.gz)`,
    /// `<output>_R2.fastq(.gz)` and `<output>_singletons.fastq(.gz)`.
    #[arg(long)]
    output: Option<String>,

    /// Force gzip compression for outputs (use `--gz` to enable)
    #[arg(long, default_value_t = false)]
    gz: bool,

    /// Gzip compression level for outputs (0-9). Lower is faster; 1 is a sensible fast default.
    #[arg(long, default_value_t = 1)]
    gz_level: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let start = Instant::now();

    match (args.input, args.p1, args.p2) {
        (Some(path), None, None) => {
            // single-end mode: trimming enabled by default (counts kept for logging)
            let reader = open_input(&path)?;
            let fq = fastq::Reader::new(BufReader::new(reader));

            // require `--output` (no stdout allowed)
            let out_name = match &args.output {
                Some(o) => o,
                None => {
                    eprintln!("Error: --output is required; stdout is not allowed");
                    std::process::exit(2);
                }
            };

            let f = File::create(out_name)?;
            let writer: Box<dyn Write> = if args.gz {
                Box::new(GzEncoder::new(
                    BufWriter::new(f),
                    Compression::new(args.gz_level),
                ))
            } else {
                Box::new(BufWriter::new(f))
            };
            let mut fqw = fastq::Writer::new(writer);

            let mut kept: u64 = 0;
            let mut dropped: u64 = 0;
            let mut read_count: u64 = 0;
            let mut base_count: u64 = 0;

            for result in fq.records() {
                let rec = result?;
                read_count += 1;
                base_count += rec.seq().len() as u64;
                if let Some((seq, qual)) =
                    trim_record(rec.qual(), rec.seq(), args.qual, args.min_len, args.window)
                {
                    // write record with same id/desc
                    fqw.write(rec.id(), rec.desc(), &seq, &qual)?;
                    kept += 1;
                } else {
                    dropped += 1;
                }
            }

            eprintln!("trimmed kept: {}  dropped: {}", kept, dropped);
            println!("reads: {}", read_count);
            println!("bases: {}", base_count);
        }
        (None, Some(p1), Some(p2)) => {
            // paired-end mode: require output base name to write R1/R2 and singletons
            let out_base = match &args.output {
                Some(o) => o.clone(),
                None => {
                    eprintln!("Error: --output is required for paired-end mode");
                    std::process::exit(2);
                }
            };

            // build output filenames based on `--gz` flag
            let (r1_name, r2_name, single_name) = io_utils::make_output_files(&out_base, args.gz);

            // open input readers for counting/processing
            let _ = open_input(&p1)?; // validate paths early
            let _ = open_input(&p2)?;

            let r1_proc = open_input(&p1)?;
            let r2_proc = open_input(&p2)?;

            let fq1 = fastq::Reader::new(BufReader::new(r1_proc));
            let fq2 = fastq::Reader::new(BufReader::new(r2_proc));

            // prepare output writers
            let make_writer = |name: &str| -> Result<Box<dyn Write>, Box<dyn Error>> {
                let f = File::create(name)?;
                if args.gz {
                    Ok(Box::new(GzEncoder::new(
                        BufWriter::new(f),
                        Compression::new(args.gz_level),
                    )))
                } else {
                    Ok(Box::new(BufWriter::new(f)))
                }
            };

            let w1 = make_writer(&r1_name)?;
            let w2 = make_writer(&r2_name)?;
            let ws = make_writer(&single_name)?;

            let mut w_r1 = fastq::Writer::new(w1);
            let mut w_r2 = fastq::Writer::new(w2);
            let mut w_s = fastq::Writer::new(ws);

            // iterate records in lock-step, handle leftovers as singletons
            let mut iter1 = fq1.records();
            let mut iter2 = fq2.records();

            let mut pairs_total: u64 = 0;
            let mut pairs_kept: u64 = 0;
            let mut pairs_dropped: u64 = 0;
            let mut singletons: u64 = 0;
            let mut read_r1: u64 = 0;
            let mut read_r2: u64 = 0;

            loop {
                match (iter1.next(), iter2.next()) {
                    (None, None) => break,
                    (Some(r1_res), Some(r2_res)) => {
                        let rec1 = r1_res?;
                        let rec2 = r2_res?;
                        read_r1 += 1;
                        read_r2 += 1;
                        pairs_total += 1;

                        let t1 = trim_record(
                            rec1.qual(),
                            rec1.seq(),
                            args.qual,
                            args.min_len,
                            args.window,
                        );
                        let t2 = trim_record(
                            rec2.qual(),
                            rec2.seq(),
                            args.qual,
                            args.min_len,
                            args.window,
                        );

                        match (t1, t2) {
                            (Some((seq1, qual1)), Some((seq2, qual2))) => {
                                w_r1.write(rec1.id(), rec1.desc(), &seq1, &qual1)?;
                                w_r2.write(rec2.id(), rec2.desc(), &seq2, &qual2)?;
                                pairs_kept += 1;
                            }
                            (Some((seq1, qual1)), None) => {
                                w_s.write(rec1.id(), rec1.desc(), &seq1, &qual1)?;
                                singletons += 1;
                            }
                            (None, Some((seq2, qual2))) => {
                                w_s.write(rec2.id(), rec2.desc(), &seq2, &qual2)?;
                                singletons += 1;
                            }
                            (None, None) => {
                                pairs_dropped += 1;
                            }
                        }
                    }
                    (Some(r1_res), None) => {
                        let rec1 = r1_res?;
                        read_r1 += 1;
                        // no partner - handle as singleton if it survives trimming
                        if let Some((seq1, qual1)) = trim_record(
                            rec1.qual(),
                            rec1.seq(),
                            args.qual,
                            args.min_len,
                            args.window,
                        ) {
                            w_s.write(rec1.id(), rec1.desc(), &seq1, &qual1)?;
                            singletons += 1;
                        }
                    }
                    (None, Some(r2_res)) => {
                        let rec2 = r2_res?;
                        read_r2 += 1;
                        if let Some((seq2, qual2)) = trim_record(
                            rec2.qual(),
                            rec2.seq(),
                            args.qual,
                            args.min_len,
                            args.window,
                        ) {
                            w_s.write(rec2.id(), rec2.desc(), &seq2, &qual2)?;
                            singletons += 1;
                        }
                    }
                }
            }

            println!("reads_R1: {}", read_r1);
            println!("reads_R2: {}", read_r2);
            println!("pairs_total: {}", pairs_total);
            println!("pairs_kept: {}", pairs_kept);
            println!("pairs_dropped: {}", pairs_dropped);
            println!("singletons: {}", singletons);
            if read_r1 != read_r2 {
                eprintln!(
                    "warning: R1 and R2 have different read counts ({} != {})",
                    read_r1, read_r2
                );
            }
        }
        _ => {
            eprintln!("Error: provide either a positional input or both --p1 and --p2");
            std::process::exit(2);
        }
    }

    let elapsed = start.elapsed();
    eprintln!("elapsed: {:.3} s", elapsed.as_secs_f64());

    Ok(())
}
