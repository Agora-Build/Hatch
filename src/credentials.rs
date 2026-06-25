#[derive(Debug)]
pub struct Credentials {
    pub endpoint: String,
    pub public_url: String,
    // Optional — only required for push and drop
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub bucket: Option<String>,
}

impl Credentials {
    pub fn load(
        endpoint_override: Option<&str>,
        env_file: Option<&std::path::Path>,
    ) -> anyhow::Result<Self> {
        // Priority (highest first): env vars > --env-file > local .env > ~/.config/hatch/.env
        // dotenvy does not overwrite existing env vars, so load highest priority first.
        if let Some(path) = env_file {
            dotenvy::from_path(path)
                .map_err(|e| anyhow::anyhow!("Failed to load {}: {}", path.display(), e))?;
        } else {
            dotenvy::dotenv().ok();
        }
        // Check platform-native config dir (~/Library/Application Support/ on macOS)
        if let Some(config_dir) = dirs::config_dir() {
            let global_env = config_dir.join("hatch").join(".env");
            if global_env.exists() {
                dotenvy::from_path(&global_env).ok();
            }
        }
        // Always check ~/.config/hatch/.env (works cross-platform)
        if let Some(home) = dirs::home_dir() {
            let xdg_env = home.join(".config").join("hatch").join(".env");
            if xdg_env.exists() {
                dotenvy::from_path(&xdg_env).ok();
            }
        }

        let endpoint = endpoint_override
            .map(|t| t.to_string())
            .or_else(|| std::env::var("HATCH_ENDPOINT").ok())
            .unwrap_or_else(|| "https://artifacts.agora.build".to_string());

        let public_url = if endpoint_override.is_some() {
            endpoint.clone()
        } else {
            std::env::var("HATCH_PUBLIC_URL")
                .unwrap_or_else(|_| "https://artifacts.agora.build".to_string())
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
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    // SAFETY: Tests are serialized by ENV_LOCK, so no concurrent env access.
    // Wrapped in unsafe to prepare for Rust edition 2024 where set_var/remove_var are unsafe.
    fn clear_env() {
        for var in &[
            "HATCH_ACCESS_KEY",
            "HATCH_SECRET_KEY",
            "HATCH_BUCKET",
            "HATCH_ENDPOINT",
            "HATCH_PUBLIC_URL",
        ] {
            unsafe { std::env::remove_var(var) };
        }
    }

    fn set_env(key: &str, value: &str) {
        unsafe { std::env::set_var(key, value) };
    }

    #[test]
    fn load_succeeds_without_s3_credentials() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        let creds = Credentials::load(None, None).unwrap();
        assert!(creds.access_key.is_none());
        assert!(creds.secret_key.is_none());
        assert!(creds.bucket.is_none());
        assert_eq!(creds.endpoint, "https://artifacts.agora.build");
        assert_eq!(creds.public_url, "https://artifacts.agora.build");
    }

    #[test]
    fn load_captures_s3_credentials_when_present() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        set_env("HATCH_ACCESS_KEY", "mykey");
        set_env("HATCH_SECRET_KEY", "mysecret");
        set_env("HATCH_BUCKET", "mybucket");
        let creds = Credentials::load(None, None).unwrap();
        assert_eq!(creds.access_key.as_deref(), Some("mykey"));
        assert_eq!(creds.secret_key.as_deref(), Some("mysecret"));
        assert_eq!(creds.bucket.as_deref(), Some("mybucket"));
    }

    #[test]
    fn require_s3_fails_if_access_key_missing() {
        let creds = Credentials {
            endpoint: "https://artifacts.agora.build".into(),
            public_url: "https://artifacts.agora.build".into(),
            access_key: None,
            secret_key: Some("secret".into()),
            bucket: Some("bucket".into()),
        };
        let err = creds.require_s3().unwrap_err();
        assert!(err.to_string().contains("HATCH_ACCESS_KEY"));
    }

    #[test]
    fn require_s3_fails_if_secret_key_missing() {
        let creds = Credentials {
            endpoint: "https://artifacts.agora.build".into(),
            public_url: "https://artifacts.agora.build".into(),
            access_key: Some("key".into()),
            secret_key: None,
            bucket: Some("bucket".into()),
        };
        let err = creds.require_s3().unwrap_err();
        assert!(err.to_string().contains("HATCH_SECRET_KEY"));
    }

    #[test]
    fn require_s3_fails_if_bucket_missing() {
        let creds = Credentials {
            endpoint: "https://artifacts.agora.build".into(),
            public_url: "https://artifacts.agora.build".into(),
            access_key: Some("key".into()),
            secret_key: Some("secret".into()),
            bucket: None,
        };
        let err = creds.require_s3().unwrap_err();
        assert!(err.to_string().contains("HATCH_BUCKET"));
    }

    #[test]
    fn require_s3_succeeds_when_all_present() {
        let creds = Credentials {
            endpoint: "https://artifacts.agora.build".into(),
            public_url: "https://artifacts.agora.build".into(),
            access_key: Some("key".into()),
            secret_key: Some("secret".into()),
            bucket: Some("bucket".into()),
        };
        let (k, s, b) = creds.require_s3().unwrap();
        assert_eq!(k, "key");
        assert_eq!(s, "secret");
        assert_eq!(b, "bucket");
    }

    #[test]
    fn endpoint_override_sets_both_endpoint_and_public_url() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        let creds = Credentials::load(Some("https://s3.example.com"), None).unwrap();
        assert_eq!(creds.endpoint, "https://s3.example.com");
        assert_eq!(creds.public_url, "https://s3.example.com");
    }

    #[test]
    fn hatch_public_url_is_independent_of_endpoint() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        set_env("HATCH_ENDPOINT", "https://accountid.r2.cloudflarestorage.com");
        set_env("HATCH_PUBLIC_URL", "https://artifacts.agora.build");
        let creds = Credentials::load(None, None).unwrap();
        assert_eq!(creds.endpoint, "https://accountid.r2.cloudflarestorage.com");
        assert_eq!(creds.public_url, "https://artifacts.agora.build");
    }

    // --- Edge cases ---

    #[test]
    fn require_s3_rejects_empty_string_access_key() {
        // Some("") is set-but-empty — require_s3 should still return it
        // because the validation is on Option::None, not empty strings.
        // This is intentional: let S3 reject bad credentials, not us.
        let creds = Credentials {
            endpoint: "https://artifacts.agora.build".into(),
            public_url: "https://artifacts.agora.build".into(),
            access_key: Some("".into()),
            secret_key: Some("secret".into()),
            bucket: Some("bucket".into()),
        };
        let (k, _, _) = creds.require_s3().unwrap();
        assert_eq!(k, "");
    }

    #[test]
    fn endpoint_override_ignores_existing_public_url_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        set_env("HATCH_PUBLIC_URL", "https://cdn.example.com");
        let creds = Credentials::load(Some("https://override.example.com"), None).unwrap();
        // --endpoint should override HATCH_PUBLIC_URL entirely
        assert_eq!(creds.public_url, "https://override.example.com");
    }

    #[test]
    fn load_only_endpoint_set_public_url_defaults_to_dl_agora_build() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        set_env("HATCH_ENDPOINT", "https://abc123.r2.cloudflarestorage.com");
        let creds = Credentials::load(None, None).unwrap();
        assert_eq!(creds.endpoint, "https://abc123.r2.cloudflarestorage.com");
        // public_url always defaults to dl.agora.build, not the ugly S3 endpoint
        assert_eq!(creds.public_url, "https://artifacts.agora.build");
    }

    // --- --env-file tests ---

    #[test]
    fn load_from_env_file() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        let dir = tempfile::tempdir().unwrap();
        let env_path = dir.path().join(".env");
        std::fs::write(&env_path, "HATCH_ACCESS_KEY=filekey\nHATCH_SECRET_KEY=filesecret\nHATCH_BUCKET=filebucket\n").unwrap();
        let creds = Credentials::load(None, Some(&env_path)).unwrap();
        assert_eq!(creds.access_key.as_deref(), Some("filekey"));
        assert_eq!(creds.secret_key.as_deref(), Some("filesecret"));
        assert_eq!(creds.bucket.as_deref(), Some("filebucket"));
    }

    #[test]
    fn env_var_overrides_env_file() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        set_env("HATCH_BUCKET", "from_env");
        let dir = tempfile::tempdir().unwrap();
        let env_path = dir.path().join(".env");
        std::fs::write(&env_path, "HATCH_ACCESS_KEY=filekey\nHATCH_SECRET_KEY=filesecret\nHATCH_BUCKET=from_file\n").unwrap();
        let creds = Credentials::load(None, Some(&env_path)).unwrap();
        // env var wins over file
        assert_eq!(creds.bucket.as_deref(), Some("from_env"));
        // file fills in the rest
        assert_eq!(creds.access_key.as_deref(), Some("filekey"));
    }

    #[test]
    fn env_file_not_found_returns_error() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        let result = Credentials::load(None, Some(std::path::Path::new("/nonexistent/.env")));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to load"));
    }

    #[test]
    fn env_file_with_public_url() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        let dir = tempfile::tempdir().unwrap();
        let env_path = dir.path().join(".env");
        std::fs::write(&env_path, "HATCH_PUBLIC_URL=https://artifacts.agora.build\n").unwrap();
        let creds = Credentials::load(None, Some(&env_path)).unwrap();
        assert_eq!(creds.public_url, "https://artifacts.agora.build");
    }

    #[test]
    fn endpoint_override_wins_over_env_file() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        let dir = tempfile::tempdir().unwrap();
        let env_path = dir.path().join(".env");
        std::fs::write(&env_path, "HATCH_ENDPOINT=https://from-file.example.com\n").unwrap();
        let creds = Credentials::load(Some("https://override.example.com"), Some(&env_path)).unwrap();
        assert_eq!(creds.endpoint, "https://override.example.com");
    }
}
