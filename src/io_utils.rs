use std::error::Error;
use std::fs::File;
use std::io::{self, Read, BufRead, BufReader};
use flate2::read::MultiGzDecoder;

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

#[cfg(test)]
mod tests {
    use super::open_input;
    use tempfile::NamedTempFile;
    use std::io::{Read, Write};
    use flate2::write::GzEncoder;
    use flate2::Compression;

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
}
