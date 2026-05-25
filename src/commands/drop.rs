use crate::path_utils::object_key;
use crate::storage::Storage;
use anyhow::Result;

fn confirm(prompt: &str) -> Result<bool> {
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() {
        anyhow::bail!("Non-interactive terminal: pass --yes to confirm");
    }
    use std::io::{BufRead, Write};
    print!("{} [y/N] ", prompt);
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().eq_ignore_ascii_case("y"))
}

/// Single-file delete (original behavior).
pub async fn run_single(
    storage: &dyn Storage,
    file: &str,
    path: &str,
    yes: bool,
) -> Result<()> {
    let key = object_key(path, file);

    if !storage.exists(&key).await? {
        anyhow::bail!("File not found: {}", key);
    }

    if !yes && !confirm(&format!("Delete {}?", key))? {
        println!("Aborted.");
        return Ok(());
    }

    storage.delete(&key).await?;
    for ext in &["md5", "sha256"] {
        if let Err(e) = storage.delete(&format!("{}.{}", key, ext)).await {
            eprintln!("Warning: failed to delete sidecar {}.{}: {}", key, ext, e);
        }
    }

    println!("Deleted: {}", key);
    Ok(())
}

/// Batch delete by prefix with optional regex filter.
pub async fn run_batch(
    storage: &dyn Storage,
    path: &str,
    filter: Option<&str>,
    dry_run: bool,
    yes: bool,
) -> Result<()> {
    let prefix = path.trim_matches('/');

    // Compile regex filter if provided
    let re = match filter {
        Some(pattern) => Some(
            regex::Regex::new(pattern)
                .map_err(|e| anyhow::anyhow!("Invalid regex '{}': {}", pattern, e))?,
        ),
        None => None,
    };

    // List all objects under prefix
    eprint!("Scanning {}...", prefix);
    let all_objects = storage.list_all(prefix).await?;
    eprintln!(" found {} objects", all_objects.len());

    // Apply regex filter
    let keys: Vec<String> = if let Some(re) = &re {
        all_objects
            .iter()
            .filter(|obj| re.is_match(&obj.key))
            .map(|obj| obj.key.clone())
            .collect()
    } else {
        all_objects.iter().map(|obj| obj.key.clone()).collect()
    };

    if keys.is_empty() {
        println!("No matching objects found.");
        return Ok(());
    }

    // Show summary
    println!("{} objects to delete", keys.len());
    let preview_count = std::cmp::min(keys.len(), 10);
    for key in &keys[..preview_count] {
        println!("  {}", key);
    }
    if keys.len() > 10 {
        println!("  ... and {} more", keys.len() - 10);
    }

    if dry_run {
        println!("\n(dry run — nothing deleted)");
        return Ok(());
    }

    if !yes && !confirm(&format!("Delete {} objects?", keys.len()))? {
        println!("Aborted.");
        return Ok(());
    }

    let deleted = storage.delete_batch(&keys).await?;
    println!("Deleted {} objects", deleted);
    Ok(())
}
