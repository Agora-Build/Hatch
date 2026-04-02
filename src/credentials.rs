pub struct Credentials {
    pub endpoint: String,
    pub public_url: String,
    // Optional — only required for push and drop
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub bucket: Option<String>,
}

impl Credentials {
    pub fn load(target_override: Option<&str>) -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let endpoint = target_override
            .map(|t| t.to_string())
            .or_else(|| std::env::var("HATCH_ENDPOINT").ok())
            .unwrap_or_else(|| "https://dl.agora.build".to_string());

        let public_url = if target_override.is_some() {
            endpoint.clone()
        } else {
            std::env::var("HATCH_PUBLIC_URL").unwrap_or_else(|_| endpoint.clone())
        };

        Ok(Credentials {
            endpoint,
            public_url,
            access_key: std::env::var("HATCH_ACCESS_KEY").ok(),
            secret_key: std::env::var("HATCH_SECRET_KEY").ok(),
            bucket: std::env::var("HATCH_BUCKET").ok(),
        })
    }

    /// Returns (access_key, secret_key, bucket) or fails with a clear error.
    /// Call this in push and drop before performing any S3 operation.
    pub fn require_s3(&self) -> anyhow::Result<(&str, &str, &str)> {
        let access_key = self.access_key.as_deref()
            .ok_or_else(|| anyhow::anyhow!("HATCH_ACCESS_KEY not set. Add it to .env or export it."))?;
        let secret_key = self.secret_key.as_deref()
            .ok_or_else(|| anyhow::anyhow!("HATCH_SECRET_KEY not set. Add it to .env or export it."))?;
        let bucket = self.bucket.as_deref()
            .ok_or_else(|| anyhow::anyhow!("HATCH_BUCKET not set. Add it to .env or export it."))?;
        Ok((access_key, secret_key, bucket))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clear_env() {
        for var in &[
            "HATCH_ACCESS_KEY",
            "HATCH_SECRET_KEY",
            "HATCH_BUCKET",
            "HATCH_ENDPOINT",
            "HATCH_PUBLIC_URL",
        ] {
            std::env::remove_var(var);
        }
    }

    #[test]
    fn load_succeeds_without_s3_credentials() {
        clear_env();
        let creds = Credentials::load(None).unwrap();
        assert!(creds.access_key.is_none());
        assert!(creds.secret_key.is_none());
        assert!(creds.bucket.is_none());
        assert_eq!(creds.endpoint, "https://dl.agora.build");
        assert_eq!(creds.public_url, "https://dl.agora.build");
    }

    #[test]
    fn load_captures_s3_credentials_when_present() {
        clear_env();
        std::env::set_var("HATCH_ACCESS_KEY", "mykey");
        std::env::set_var("HATCH_SECRET_KEY", "mysecret");
        std::env::set_var("HATCH_BUCKET", "mybucket");
        let creds = Credentials::load(None).unwrap();
        assert_eq!(creds.access_key.as_deref(), Some("mykey"));
        assert_eq!(creds.secret_key.as_deref(), Some("mysecret"));
        assert_eq!(creds.bucket.as_deref(), Some("mybucket"));
    }

    #[test]
    fn require_s3_fails_if_access_key_missing() {
        clear_env();
        std::env::set_var("HATCH_SECRET_KEY", "secret");
        std::env::set_var("HATCH_BUCKET", "bucket");
        let creds = Credentials::load(None).unwrap();
        let err = creds.require_s3().unwrap_err();
        assert!(err.to_string().contains("HATCH_ACCESS_KEY"));
    }

    #[test]
    fn require_s3_fails_if_secret_key_missing() {
        clear_env();
        std::env::set_var("HATCH_ACCESS_KEY", "key");
        std::env::set_var("HATCH_BUCKET", "bucket");
        let creds = Credentials::load(None).unwrap();
        let err = creds.require_s3().unwrap_err();
        assert!(err.to_string().contains("HATCH_SECRET_KEY"));
    }

    #[test]
    fn require_s3_fails_if_bucket_missing() {
        clear_env();
        std::env::set_var("HATCH_ACCESS_KEY", "key");
        std::env::set_var("HATCH_SECRET_KEY", "secret");
        let creds = Credentials::load(None).unwrap();
        let err = creds.require_s3().unwrap_err();
        assert!(err.to_string().contains("HATCH_BUCKET"));
    }

    #[test]
    fn require_s3_succeeds_when_all_present() {
        clear_env();
        std::env::set_var("HATCH_ACCESS_KEY", "key");
        std::env::set_var("HATCH_SECRET_KEY", "secret");
        std::env::set_var("HATCH_BUCKET", "bucket");
        let creds = Credentials::load(None).unwrap();
        let (k, s, b) = creds.require_s3().unwrap();
        assert_eq!(k, "key");
        assert_eq!(s, "secret");
        assert_eq!(b, "bucket");
    }

    #[test]
    fn target_override_sets_both_endpoint_and_public_url() {
        clear_env();
        let creds = Credentials::load(Some("https://s3.example.com")).unwrap();
        assert_eq!(creds.endpoint, "https://s3.example.com");
        assert_eq!(creds.public_url, "https://s3.example.com");
    }

    #[test]
    fn hatch_public_url_is_independent_of_endpoint() {
        clear_env();
        std::env::set_var("HATCH_ENDPOINT", "https://accountid.r2.cloudflarestorage.com");
        std::env::set_var("HATCH_PUBLIC_URL", "https://dl.agora.build");
        let creds = Credentials::load(None).unwrap();
        assert_eq!(creds.endpoint, "https://accountid.r2.cloudflarestorage.com");
        assert_eq!(creds.public_url, "https://dl.agora.build");
    }
}
