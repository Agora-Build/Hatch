use std::path::Path;
use anyhow::Result;
use md5::Digest as _;

pub struct Checksums {
    pub md5: String,
    pub sha256: String,
}

pub fn compute(path: &Path) -> Result<Checksums> {
    use std::fs::File;
    use std::io::{BufReader, Read};

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut md5_hasher = md5::Md5::new();
    let mut sha256_hasher = sha2::Sha256::new();
    let mut buf = vec![0u8; 65536];

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        md5::Digest::update(&mut md5_hasher, &buf[..n]);
        sha2::Digest::update(&mut sha256_hasher, &buf[..n]);
    }

    Ok(Checksums {
        md5: format!("{:x}", md5::Digest::finalize(md5_hasher)),
        sha256: format!("{:x}", sha2::Digest::finalize(sha256_hasher)),
    })
}

pub fn format_line(digest: &str, filename: &str) -> String {
    format!("{}  {}", digest, filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn tmp(content: &[u8]) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content).unwrap();
        f
    }

    #[test]
    fn compute_md5_of_known_content() {
        let f = tmp(b"hello");
        let cs = compute(f.path()).unwrap();
        assert_eq!(cs.md5, "5d41402abc4b2a76b9719d911017c592");
    }

    #[test]
    fn compute_sha256_of_known_content() {
        let f = tmp(b"hello");
        let cs = compute(f.path()).unwrap();
        assert_eq!(cs.sha256, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    }

    #[test]
    fn format_line_matches_standard_checksum_tool_output() {
        assert_eq!(format_line("abc123", "file.zip"), "abc123  file.zip");
    }

    #[test]
    fn compute_handles_empty_file() {
        let f = tmp(b"");
        let cs = compute(f.path()).unwrap();
        assert_eq!(cs.md5, "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(cs.sha256, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    // --- Edge cases ---

    #[test]
    fn compute_large_file_spans_multiple_buffer_reads() {
        // 200KB — exceeds the 64KB internal buffer, forces multiple read() loops
        let data = vec![0xABu8; 200 * 1024];
        let f = tmp(&data);
        let cs = compute(f.path()).unwrap();
        // Just verify it produces 32-char hex (MD5) and 64-char hex (SHA256)
        assert_eq!(cs.md5.len(), 32);
        assert_eq!(cs.sha256.len(), 64);
        assert!(cs.md5.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(cs.sha256.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn compute_nonexistent_file_returns_error() {
        let result = compute(Path::new("/tmp/hatch_nonexistent_file_42"));
        assert!(result.is_err());
    }

    #[test]
    fn format_line_with_filename_containing_spaces() {
        assert_eq!(
            format_line("abc123", "my file (1).zip"),
            "abc123  my file (1).zip"
        );
    }

    #[test]
    fn compute_single_byte_file() {
        let f = tmp(b"\x00");
        let cs = compute(f.path()).unwrap();
        // MD5 of a single null byte
        assert_eq!(cs.md5, "93b885adfe0da089cdf634904fd59f71");
    }

    #[test]
    fn compute_exactly_64kb_file() {
        // Exactly one buffer size — tests the boundary between 1 and 2 reads
        let data = vec![0x42u8; 65536];
        let f = tmp(&data);
        let cs = compute(f.path()).unwrap();
        assert_eq!(cs.md5.len(), 32);
        assert_eq!(cs.sha256.len(), 64);
    }
}
