# rusty — small Rust NGS learning project

This repo is a minimal Rust starter for NGS tasks. It includes a tiny CLI that counts reads and bases from a FASTQ/FASTQ.GZ file.

Build:
```sh
cd rusty
cargo build --release
```

Run (example):
```sh
cargo run -- sample.fastq
cargo run -- sample.fastq.gz
cat sample.fastq | cargo run -- -
```

Next ideas:
- Add multi-threaded parsing with `rayon`.
- Implement k-mer counting or simple aligner examples.
- Add tests and CI (GitHub Actions).
