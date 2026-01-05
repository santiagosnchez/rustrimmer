use flate2::read::MultiGzDecoder;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

pub fn open_input(path: &str) -> Result<Box<dyn Read>, Box<dyn Error>> {
    if path == "-" {
        let mut br = BufReader::new(io::stdin());
        let buf = br.fill_buf()?;
        let is_gz = buf.len() >= 2 && buf[0] == 0x1f && buf[1] == 0x8b;
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

/// Given an output base name and gzip flag, return file paths for R1, R2 and singletons.
pub fn make_output_files(base: &str, gz: bool, zstd: bool) -> (String, String, String) {
    if gz {
        (
            format!("{}{}_R1.fastq.gz", base, ""),
            format!("{}{}_R2.fastq.gz", base, ""),
            format!("{}{}_singletons.fastq.gz", base, ""),
        )
    } else if zstd {
        (
            format!("{}{}_R1.fastq.zst", base, ""),
            format!("{}{}_R2.fastq.zst", base, ""),
            format!("{}{}_singletons.fastq.zst", base, ""),
        )
    } else {
        (
            format!("{}{}_R1.fastq", base, ""),
            format!("{}{}_R2.fastq", base, ""),
            format!("{}{}_singletons.fastq", base, ""),
        )
    }
}

/// Return plain (non-.zst) filenames when `zstd` is true, otherwise return the
/// provided names cloned as owned `String`s.
pub fn make_plain_filenames(
    r1: &str,
    r2: &str,
    single: &str,
    zstd: bool,
) -> (String, String, String) {
    if zstd {
        (
            r1.trim_end_matches(".zst").to_string(),
            r2.trim_end_matches(".zst").to_string(),
            single.trim_end_matches(".zst").to_string(),
        )
    } else {
        (r1.to_string(), r2.to_string(), single.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::open_input;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::{Read, Write};
    use tempfile::NamedTempFile;

    #[test]
    fn open_plain_file_reads_contents() -> Result<(), Box<dyn std::error::Error>> {
        let mut tmp = NamedTempFile::new()?;
        write!(tmp, "hello-plain")?;
        let path = tmp.path().to_str().unwrap().to_string();

        let mut reader = open_input(&path)?;
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        assert_eq!(buf, "hello-plain");
        Ok(())
    }

    #[test]
    fn open_gz_file_reads_decompressed() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = NamedTempFile::new()?;
        let path = tmp.path().to_str().unwrap().to_string();

        // create gz content
        {
            let f = std::fs::File::create(&path)?;
            let mut gz = GzEncoder::new(f, Compression::default());
            write!(gz, "hello-gz")?;
            gz.finish()?;
        }

        let mut reader = open_input(&path)?;
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        assert_eq!(buf, "hello-gz");
        Ok(())
    }

    #[test]
    fn open_stdin_gz_reads_decompressed() -> Result<(), Box<dyn std::error::Error>> {
        // create gz content in memory
        let mut gz_data: Vec<u8> = Vec::new();
        {
            let mut gz = GzEncoder::new(&mut gz_data, Compression::default());
            write!(gz, "hello-stdin-gz")?;
            gz.finish()?;
        }
        // write gz content to a temporary file and open it (avoids blocking on real stdin)
        let mut tmp = NamedTempFile::new()?;
        tmp.write_all(&gz_data)?;
        let path = tmp.path().to_str().unwrap().to_string();
        let mut reader = open_input(&path)?;
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        assert_eq!(buf, "hello-stdin-gz");
        Ok(())
    }

    #[test]
    fn make_output_files_gz() {
        let (r1, r2, single) = super::make_output_files("output", true, false);
        assert_eq!(r1, "output_R1.fastq.gz");
        assert_eq!(r2, "output_R2.fastq.gz");
        assert_eq!(single, "output_singletons.fastq.gz");
    }
}
