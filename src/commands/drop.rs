use crate::storage::Storage;
use anyhow::Result;

fn object_key(path: &str, filename: &str) -> String {
    format!("{}/{}", path.trim_matches('/'), filename)
}

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
    // Best-effort: delete sidecars; ignore errors if they don't exist
    let _ = storage.delete(&format!("{}.md5", key)).await;
    let _ = storage.delete(&format!("{}.sha256", key)).await;

    println!("Deleted: {}", key);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_key_strips_path_slashes() {
        assert_eq!(object_key("/release/v1/", "app.zip"), "release/v1/app.zip");
        assert_eq!(object_key("release/v1", "app.zip"), "release/v1/app.zip");
    }
}
