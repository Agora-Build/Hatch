pub mod s3;

use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct StorageObject {
    pub key: String,
    pub size: u64,
    pub last_modified: String,
}

#[derive(Debug)]
pub struct ListResult {
    pub objects: Vec<StorageObject>,
    pub is_truncated: bool,
}

#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    /// Upload a file from disk by path (streaming).
    async fn upload(&self, key: &str, path: &Path) -> Result<()>;
    /// Upload small in-memory content (used for sidecar files).
    async fn upload_bytes(&self, key: &str, content: &[u8]) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list(&self, prefix: &str, max_keys: u32) -> Result<ListResult>;
    async fn exists(&self, key: &str) -> Result<bool>;
}
