use crate::path_utils::object_key;
use crate::storage::Storage;
use anyhow::Result;

pub async fn run(
    storage: &dyn Storage,
    file: &str,
    path: &str,
    yes: bool,
) -> Result<()> {
    use std::io::IsTerminal;

    let key = object_key(path, file);

    if !storage.exists(&key).await? {
        anyhow::bail!("File not found: {}", key);
    }

    if !yes {
        if !std::io::stdin().is_terminal() {
            anyhow::bail!(
                "Non-interactive terminal: pass --yes to confirm deletion of {}",
                key
            );
        }
        use std::io::{BufRead, Write};
        print!("Delete {}? [y/N] ", key);
        std::io::stdout().flush()?;
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line)?;
        if !line.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    storage.delete(&key).await?;
    // Best-effort sidecar cleanup — warn on unexpected failures
    for ext in &["md5", "sha256"] {
        if let Err(e) = storage.delete(&format!("{}.{}", key, ext)).await {
            eprintln!("Warning: failed to delete sidecar {}.{}: {}", key, ext, e);
        }
    }

    println!("Deleted: {}", key);
    Ok(())
}

// Path utility tests are in src/path_utils.rs
