use crate::rate_limiter::RateLimiter;
use crate::storage::Storage;
use anyhow::Result;
use serde::Serialize;

const MAX_KEYS_LIMIT: i32 = 500;
const REQUESTS_PER_SEC: u32 = 5;

#[derive(Serialize)]
struct JsonObject {
    key: String,
    size: u64,
    last_modified: String,
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub async fn run(
    storage: &dyn Storage,
    path: &str,
    max_keys: i32,
    json: bool,
) -> Result<()> {
    let prefix = path.trim_matches('/');
    let capped = std::cmp::min(max_keys, MAX_KEYS_LIMIT);
    let mut rl = RateLimiter::new(REQUESTS_PER_SEC);

    rl.acquire().await;
    let objects = storage.list(prefix, capped).await.map_err(|e| {
        // Translate S3 403/auth errors into a helpful message
        let msg = e.to_string().to_lowercase();
        if msg.contains("403") || msg.contains("forbidden") || msg.contains("access denied") {
            anyhow::anyhow!(
                "Listing requires credentials — set HATCH_ACCESS_KEY, HATCH_SECRET_KEY, and HATCH_BUCKET."
            )
        } else {
            e
        }
    })?;

    if json {
        let items: Vec<JsonObject> = objects
            .iter()
            .map(|o| JsonObject {
                key: o.key.clone(),
                size: o.size,
                last_modified: o.last_modified.clone(),
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }

    if objects.is_empty() {
        println!("No files found at {}", path);
        return Ok(());
    }

    println!("{:<60} {:>12}  {}", "KEY", "SIZE", "LAST MODIFIED");
    println!("{}", "-".repeat(90));
    for obj in &objects {
        println!(
            "{:<60} {:>12}  {}",
            obj.key,
            format_size(obj.size),
            obj.last_modified
        );
    }
    println!("\n{} object(s)", objects.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_keys_is_clamped_to_limit() {
        assert_eq!(std::cmp::min(9999, MAX_KEYS_LIMIT), 500);
        assert_eq!(std::cmp::min(50, MAX_KEYS_LIMIT), 50);
        assert_eq!(std::cmp::min(501, MAX_KEYS_LIMIT), 500);
    }

    #[test]
    fn format_size_picks_correct_unit() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    // --- Edge cases ---

    #[test]
    fn format_size_zero_bytes() {
        assert_eq!(format_size(0), "0 B");
    }

    #[test]
    fn format_size_just_below_kb() {
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn format_size_just_above_kb() {
        assert_eq!(format_size(1025), "1.0 KB");
    }

    #[test]
    fn format_size_large_gb() {
        // 10 GB
        assert_eq!(format_size(10 * 1024 * 1024 * 1024), "10.0 GB");
    }

    #[test]
    fn format_size_fractional_mb() {
        // 1.5 MB = 1572864 bytes
        assert_eq!(format_size(1572864), "1.5 MB");
    }

    #[test]
    fn max_keys_clamping_at_exact_limit() {
        assert_eq!(std::cmp::min(500, MAX_KEYS_LIMIT), 500);
    }
}
