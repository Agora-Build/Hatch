use crate::path_utils::{object_key, build_public_url};
use crate::storage::Storage;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

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

// Path utility tests are in src/path_utils.rs
