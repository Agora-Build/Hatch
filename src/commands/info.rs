use anyhow::Result;

fn build_url(public_url: &str, path: &str, filename: &str) -> String {
    format!(
        "{}/{}/{}",
        public_url.trim_end_matches('/'),
        path.trim_matches('/'),
        filename
    )
}

pub async fn run(
    public_url: &str,
    path: &str,
    file: &str,
) -> Result<()> {
    let url = build_url(public_url, path, file);
    let client = reqwest::Client::new();

    // HEAD request for metadata
    let head = client
        .head(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to reach {}: {}", url, e))?;

    if head.status() == reqwest::StatusCode::NOT_FOUND {
        anyhow::bail!("File not found: {}", url);
    }
    if !head.status().is_success() {
        anyhow::bail!("Server returned {} for {}", head.status(), url);
    }

    let size = head
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .map(|n| format!("{} bytes", n))
        .unwrap_or_else(|| "(unknown)".to_string());

    let last_modified = head
        .headers()
        .get(reqwest::header::LAST_MODIFIED)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("(unknown)")
        .to_string();

    let etag = head
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("(unknown)")
        .trim_matches('"')
        .to_string();

    println!("URL:           {}", url);
    println!("Size:          {}", size);
    println!("Last Modified: {}", last_modified);
    println!("ETag:          {}", etag);

    // GET sidecar files
    for ext in &["md5", "sha256"] {
        let sidecar_url = format!("{}.{}", url, ext);
        match client.get(&sidecar_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let body = resp.text().await.unwrap_or_default();
                let digest = body.split_whitespace().next().unwrap_or("(empty)");
                println!("{:<15}{}", format!("{}:", ext.to_uppercase()), digest);
            }
            _ => println!("{:<15}(not available)", format!("{}:", ext.to_uppercase())),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_url_constructs_correct_path() {
        assert_eq!(
            build_url("https://dl.agora.build", "/release/v1", "app.zip"),
            "https://dl.agora.build/release/v1/app.zip"
        );
        assert_eq!(
            build_url("https://dl.agora.build/", "release/v1/", "app.zip"),
            "https://dl.agora.build/release/v1/app.zip"
        );
    }

    #[test]
    fn build_url_works_with_custom_target() {
        assert_eq!(
            build_url("https://s3.example.com", "/release/v1", "file.zip"),
            "https://s3.example.com/release/v1/file.zip"
        );
    }
}
