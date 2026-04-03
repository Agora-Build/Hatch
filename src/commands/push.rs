use crate::storage::Storage;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

pub fn normalize_prefix(path: &str) -> String {
    path.trim_matches('/').to_string()
}

pub fn object_key(path: &str, filename: &str) -> String {
    format!("{}/{}", normalize_prefix(path), filename)
}

pub fn build_public_url(base: &str, path: &str, filename: &str) -> String {
    format!(
        "{}/{}/{}",
        base.trim_end_matches('/'),
        normalize_prefix(path),
        filename
    )
}

pub async fn run(
    storage: &dyn Storage,
    public_url_base: &str,
    file: &Path,
    path: &str,
    force: bool,
) -> Result<()> {
    if !file.exists() {
        anyhow::bail!("File not found: {}", file.display());
    }

    let filename = file
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename: {}", file.display()))?;

    let key = object_key(path, filename);

    if !force && storage.exists(&key).await? {
        anyhow::bail!(
            "File already exists at {} — use --force to overwrite",
            key
        );
    }

    let checksums = crate::checksum::compute(file)?;

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    pb.set_message(format!("Uploading {}...", filename));
    storage.upload(&key, file).await?;

    let url = build_public_url(public_url_base, path, filename);

    pb.set_message(format!("Uploading {}.md5...", filename));
    let md5_content = crate::checksum::format_line(&checksums.md5, filename);
    if let Err(e) = storage.upload_bytes(&format!("{}.md5", key), md5_content.as_bytes()).await {
        pb.finish_and_clear();
        eprintln!("Warning: main file uploaded to {}", url);
        eprintln!("Failed to upload {}.md5 sidecar: {}", filename, e);
        anyhow::bail!("Sidecar upload failed — main file is at {}", url);
    }

    pb.set_message(format!("Uploading {}.sha256...", filename));
    let sha256_content = crate::checksum::format_line(&checksums.sha256, filename);
    if let Err(e) = storage.upload_bytes(&format!("{}.sha256", key), sha256_content.as_bytes()).await {
        pb.finish_and_clear();
        eprintln!("Warning: main file uploaded to {}", url);
        eprintln!("Failed to upload {}.sha256 sidecar: {}", filename, e);
        anyhow::bail!("Sidecar upload failed — main file is at {}", url);
    }

    pb.finish_and_clear();

    println!("{}", url);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_public_url_trims_slashes_correctly() {
        assert_eq!(
            build_public_url("https://dl.agora.build", "/release/v1/", "app.zip"),
            "https://dl.agora.build/release/v1/app.zip"
        );
        assert_eq!(
            build_public_url("https://dl.agora.build/", "release/v1", "app.zip"),
            "https://dl.agora.build/release/v1/app.zip"
        );
    }

    #[test]
    fn normalize_prefix_strips_leading_and_trailing_slashes() {
        assert_eq!(normalize_prefix("/release/v1/"), "release/v1");
        assert_eq!(normalize_prefix("release/v1"), "release/v1");
        assert_eq!(normalize_prefix("/release/v1"), "release/v1");
    }

    #[test]
    fn object_key_combines_prefix_and_filename() {
        assert_eq!(object_key("/release/v1", "app.zip"), "release/v1/app.zip");
        assert_eq!(object_key("release/v1/", "app.zip"), "release/v1/app.zip");
    }

    // --- Edge cases ---

    #[test]
    fn normalize_prefix_empty_string() {
        assert_eq!(normalize_prefix(""), "");
    }

    #[test]
    fn normalize_prefix_just_slashes() {
        assert_eq!(normalize_prefix("///"), "");
    }

    #[test]
    fn normalize_prefix_deeply_nested() {
        assert_eq!(
            normalize_prefix("/a/b/c/d/e/f/"),
            "a/b/c/d/e/f"
        );
    }

    #[test]
    fn object_key_with_empty_path() {
        // Empty path produces "/filename" — this is technically valid S3 key
        assert_eq!(object_key("", "app.zip"), "/app.zip");
    }

    #[test]
    fn build_public_url_with_port() {
        assert_eq!(
            build_public_url("https://localhost:9000", "/release/v1", "app.zip"),
            "https://localhost:9000/release/v1/app.zip"
        );
    }

    #[test]
    fn build_public_url_with_empty_path() {
        assert_eq!(
            build_public_url("https://dl.agora.build", "", "app.zip"),
            "https://dl.agora.build//app.zip"
        );
    }

    #[test]
    fn object_key_filename_with_special_chars() {
        assert_eq!(
            object_key("/release/v1", "my app (v2.0).tar.gz"),
            "release/v1/my app (v2.0).tar.gz"
        );
    }
}
