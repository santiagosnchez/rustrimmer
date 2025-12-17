# rusty — small Rust NGS learning project

This repo is a minimal Rust starter for NGS tasks. It includes a tiny CLI that counts reads and bases from a FASTQ/FASTQ.GZ file.

Build:
```bash
cargo build --release
```

Run with Cargo (example):
```bash
cargo run -- --p1 tests/sample_R1.fastq.gz --p2 tests/sample_R2.fastq.gz
# reads_R1: 100
# bases_R1: 5000
# reads_R2: 100
# bases_R2: 5000
# pairs: 100
```

Next ideas:
- Add multi-threaded parsing with `rayon`.
- Implement k-mer counting or simple aligner examples.
- Add tests and CI (GitHub Actions).

**Release build & distribution**

- Build an optimized release binary:

```bash
cargo build --release
```

- The resulting binary is at `target/release/rusty`.

- Help example:

```bash
./target/release/rusty --help
```

- Run example:

```bash
./target/release/rusty --p1 tests/sample_R1.fastq.gz --p2 tests/sample_R2.fastq.gz
```

- Build a static-musl binary for portability:

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
# binary: target/x86_64-unknown-linux-musl/release/rusty
```

- Further shrink the binary (optional): use `strip`, `upx`, or `cargo-strip`:

```sh
# remove symbols
strip target/release/rusty
# or compress
upx --best target/release/rusty
```
