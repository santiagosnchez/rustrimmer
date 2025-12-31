# rustrimmer — A Rust-based FASTQ read-trimmer

This repo is intended as a minimal Rust starter for NGS tasks. It includes a CLI for trimming bad quality bases from the ends of reads in `fastq` files.

## **Release build & distribution**

Build:
```bash
cargo build --release
```

Check args:
```bash
./target/release/rustrimmer --help
# Simple FASTQ quality trimmer: removes low-quality bases from read ends using sliding window approach

# Usage: rustrimmer [OPTIONS] [INPUT]

# Arguments:
#   [INPUT]  Input FASTQ (use '-' for stdin). Supports .gz compressed files. Provide either a single input or both `--p1` and `--p2` for paired-end files

# Options:
#       --p1 <P1>            Paired-end R1 (e.g. sample_R1.fastq or .fastq.gz)
#       --p2 <P2>            Paired-end R2 (e.g. sample_R2.fastq or .fastq.gz)
#       --qual <QUAL>        Quality threshold (Phred) for trimming ends; default 20 [default: 20]
#       --min-len <MIN_LEN>  Minimum length to keep a read after trimming; default 30 [default: 30]
#       --window <WINDOW>    Sliding window size for trimming; use 1 to check single-base quality (default) [default: 1]
#       --output <OUTPUT>    Output file (single) or base name for paired outputs (required). For paired mode this will create `<output>_R1.fastq(.gz)`, `<output>_R2.fastq(.gz)` and `<output>_singletons.fastq(.gz)`.
#       --gz                  Force gzip compression for outputs (use to create .gz files regardless of output name)
#       --gz-level <LEVEL>    Gzip compression level (0-9). Higher gives better compression; 3 is a sensible default. [default: 3]
#   -h, --help               Print help
#   -V, --version            Print version
```

## **Generate a test dataset**

```bash
python3 tests/generate_test_fastq.py --number 100 --bad_fraction 0.3 > tests/sample_R1.fastq
python3 tests/generate_test_fastq.py --number 100 --bad_fraction 0.3 > tests/sample_R2.fastq
```

## **Run code**

Run with Cargo (examples):
```bash
# Paired-end (requires --output)
./target/release/rustrimmer --p1 tests/sample_R1.fastq --p2 tests/sample_R2.fastq --output tests/result
# Single-end (requires --output)
./target/release/rustrimmer tests/sample_R1.fastq --output tests/result_single
# Force gzip compression for outputs
./target/release/rustrimmer --p1 tests/sample_R1.fastq --p2 tests/sample_R2.fastq --output tests/result --gz --gz-level 6
# Use zstd compression (opt-in, faster but less universally supported)
./target/release/rustrimmer --p1 tests/sample_R1.fastq --p2 tests/sample_R2.fastq --output tests/result --zstd --zstd-level 3
```

Output files:
```bash
ls ./tests/result*
# ./tests/result_R1.fastq
# ./tests/result_R2.fastq
# ./tests/result_singletons.fastq
# When using `--gz` the output files will end with `.fastq.gz` (for example `tests/result_R1.fastq.gz`).
# When using `--zstd` the output files will end with `.fastq.zst` (for example `tests/result_R1.fastq.zst`).
```

Compression notes:
- **Default:** gzip is enabled by default for outputs (`--gz` is on by default) for maximum downstream compatibility.
- **zstd (optional):** Use `--zstd` for faster compression and smaller files; this is opt-in because not all bioinformatics tools accept `.zst` compressed FASTQ files.
- **Compatibility:** `.fastq.gz` is widely supported. `.fastq.zst` is a zstd-compressed FASTQ — many tools can read it (via `zstdcat` or libraries that support zstd) but it is not as universally accepted as gzip. If you need random-access/indexable FASTQ (e.g., htslib/tabix workflows), consider BGZF or produce a gzip output for compatibility.

If you plan to benchmark compression speed/size, prefer `--zstd --zstd-level <n>` for faster runs and smaller files; a sensible default is `--zstd-level 3`.
