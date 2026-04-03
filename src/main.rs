mod checksum;
mod cli;
mod commands;
mod credentials;
mod storage;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let creds = credentials::Credentials::load(cli.target.as_deref())?;

    match cli.command {
        Commands::Push { file, path, force } => {
            let storage = storage::s3::S3Client::new_authenticated(&creds).await?;
            commands::push::run(&storage, &creds.public_url, &file, &path, force).await?;
        }
        Commands::Drop { file, path, yes } => {
            let storage = storage::s3::S3Client::new_authenticated(&creds).await?;
            commands::drop::run(&storage, &file, &path, yes).await?;
        }
        Commands::List { path, max_keys, json } => {
            // Use authenticated client if credentials present, otherwise anonymous
            let storage: Box<dyn storage::Storage> =
                if creds.access_key.is_some() && creds.secret_key.is_some() && creds.bucket.is_some() {
                    Box::new(storage::s3::S3Client::new_authenticated(&creds).await?)
                } else {
                    let bucket = creds.bucket.as_deref().unwrap_or_else(|| {
                        eprintln!("Warning: HATCH_BUCKET not set — listing may fail.");
                        ""
                    });
                    Box::new(storage::s3::S3Client::new_anonymous(&creds.endpoint, bucket).await?)
                };
            commands::list::run(storage.as_ref(), &path, max_keys, json).await?;
        }
        Commands::Info { file, path } => {
            // HTTP only — no S3 client needed
            commands::info::run(&creds.public_url, &path, &file).await?;
        }
    }

    Ok(())
}
