use clap::Parser;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = rustrimmer::Args::parse();
    if let Err(e) = rustrimmer::run(args) {
        eprintln!("{}", e);
        std::process::exit(2);
    }
    Ok(())
}
