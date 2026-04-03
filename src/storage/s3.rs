use super::{ListResult, Storage, StorageObject};
use crate::credentials::Credentials;
use anyhow::{Context, Result};
use aws_sdk_s3::primitives::ByteStream;
use std::path::Path;

pub struct S3Client {
    client: aws_sdk_s3::Client,
    bucket: String,
}

impl S3Client {
    pub async fn new_authenticated(creds: &Credentials) -> Result<Self> {
        let (access_key, secret_key, bucket) = creds.require_s3()?;

        let aws_creds = aws_credential_types::Credentials::new(
            access_key,
            secret_key,
            None,
            None,
            "hatch",
        );

        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .credentials_provider(
                aws_credential_types::provider::SharedCredentialsProvider::new(aws_creds),
            )
            .endpoint_url(&creds.endpoint)
            .region(aws_config::Region::new("auto"))
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .force_path_style(true)
            .build();

        Ok(S3Client {
            client: aws_sdk_s3::Client::from_conf(s3_config),
            bucket: bucket.to_string(),
        })
    }

    pub async fn new_anonymous(endpoint: &str, bucket: &str) -> Result<Self> {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .no_credentials()
            .endpoint_url(endpoint)
            .region(aws_config::Region::new("auto"))
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .force_path_style(true)
            .build();

        Ok(S3Client {
            client: aws_sdk_s3::Client::from_conf(s3_config),
            bucket: bucket.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl Storage for S3Client {
    async fn upload(&self, key: &str, path: &Path) -> Result<()> {
        let content_length = path.metadata()
            .with_context(|| format!("Cannot stat file: {}", path.display()))?
            .len();
        let body = ByteStream::from_path(path)
            .await
            .with_context(|| format!("Cannot read file: {}", path.display()))?;
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .content_length(content_length as i64)
            .send()
            .await
            .with_context(|| format!("Failed to upload {}", key))?;
        Ok(())
    }

    async fn upload_bytes(&self, key: &str, content: &[u8]) -> Result<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(content.to_vec()))
            .content_length(content.len() as i64)
            .send()
            .await
            .with_context(|| format!("Failed to upload {}", key))?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .with_context(|| format!("Failed to delete {}", key))?;
        Ok(())
    }

    async fn list(&self, prefix: &str, max_keys: u32) -> Result<ListResult> {
        let resp = self.client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(prefix)
            .max_keys(max_keys as i32)
            .send()
            .await
            .with_context(|| format!("Failed to list objects under {}", prefix))?;

        let is_truncated = resp.is_truncated().unwrap_or(false);

        let objects = resp
            .contents()
            .iter()
            .map(|obj| StorageObject {
                key: obj.key().unwrap_or("").to_string(),
                size: obj.size().unwrap_or(0) as u64,
                last_modified: obj
                    .last_modified()
                    .map(|t| t.to_string())
                    .unwrap_or_default(),
            })
            .collect();

        Ok(ListResult { objects, is_truncated })
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        use aws_sdk_s3::error::SdkError;
        match self.client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(SdkError::ServiceError(e)) if e.err().is_not_found() => Ok(false),
            Err(e) => Err(anyhow::anyhow!("S3 error checking existence of {}: {:?}", key, e)),
        }
    }
}
