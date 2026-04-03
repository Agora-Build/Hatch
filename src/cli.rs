use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "hatch",
    version,
    about = "Secure, versioned file release CLI for S3-compatible storage",
    long_about = "Hatch uploads, manages, and distributes files in versioned release paths\n\
on S3-compatible storage, with automatic checksum generation.\n\
\n\
CREDENTIALS\n\
  Add these to a .env file in your working directory, or export as env vars:\n\
\n\
  HATCH_ACCESS_KEY=<key>       Required for: push, drop\n\
  HATCH_SECRET_KEY=<secret>    Required for: push, drop\n\
  HATCH_BUCKET=<bucket>        Required for: push, drop\n\
  HATCH_ENDPOINT=<url>         S3 API endpoint  (default: https://dl.agora.build)\n\
  HATCH_PUBLIC_URL=<url>       Public CDN URL   (default: same as HATCH_ENDPOINT)\n\
\n\
  'list' and 'info' work without credentials:\n\
    list  — tries anonymous S3; falls back with a helpful error if bucket is private\n\
    info  — uses HTTP HEAD/GET against HATCH_PUBLIC_URL, no auth required\n\
\n\
CLOUDFLARE R2 SETUP\n\
  1. Cloudflare dashboard → R2 Object Storage → Create bucket\n\
     Note your bucket name (e.g. 'releases')\n\
  2. R2 → Manage API Tokens → Create API Token\n\
     Permissions: Object Read & Write on your bucket\n\
     Copy the Access Key ID → HATCH_ACCESS_KEY\n\
     Copy the Secret Access Key → HATCH_SECRET_KEY\n\
  3. R2 → your bucket → Settings → S3 API\n\
     Copy the endpoint URL → HATCH_ENDPOINT\n\
     (format: https://<ACCOUNT_ID>.r2.cloudflarestorage.com)\n\
  4. Optional: connect a custom domain for public CDN access\n\
     R2 → your bucket → Settings → Public access → Connect domain\n\
     Set your domain (e.g. https://dl.example.com) → HATCH_PUBLIC_URL\n\
\n\
OTHER S3-COMPATIBLE STORAGE\n\
  Set HATCH_ENDPOINT to any S3-compatible API URL (e.g. https://s3.amazonaws.com),\n\
  HATCH_BUCKET to your bucket name, and credentials as above.\n\
  Use --target <endpoint> per-command to override the endpoint without changing .env.\n\
\n\
EXAMPLE .env\n\
  HATCH_ACCESS_KEY=abc123def456\n\
  HATCH_SECRET_KEY=xyz789secret\n\
  HATCH_BUCKET=releases\n\
  HATCH_ENDPOINT=https://abc123.r2.cloudflarestorage.com\n\
  HATCH_PUBLIC_URL=https://dl.agora.build\n\
\n\
RELEASE PATH CONVENTION\n\
  /release/<product>_<version>_<date>_<build>/\n\
  e.g. /release/myapp_v1.2.0_20260402_build42/"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Override the S3-compatible endpoint (overrides HATCH_ENDPOINT and HATCH_PUBLIC_URL)
    #[arg(long, global = true)]
    pub target: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Upload a file to a release path
    Push {
        /// Local file to upload
        file: std::path::PathBuf,
        /// Release path prefix (e.g. /release/product_v1_20260402_build1)
        #[arg(long)]
        path: String,
        /// Overwrite the file if it already exists
        #[arg(long)]
        force: bool,
    },
    /// Delete a file from a release path (requires confirmation)
    Drop {
        /// Filename to delete
        file: String,
        /// Release path prefix
        #[arg(long)]
        path: String,
        /// Skip the confirmation prompt (for CI use)
        #[arg(long)]
        yes: bool,
    },
    /// List files at a release path
    List {
        /// Release path prefix
        #[arg(long)]
        path: String,
        /// Maximum number of results (default: 100, max: 1000)
        #[arg(long, default_value = "100", value_parser = clap::value_parser!(i32).range(1..=1000))]
        max_keys: i32,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show metadata and checksums for a file
    Info {
        /// Filename
        file: String,
        /// Release path prefix
        #[arg(long)]
        path: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_push_command() {
        let cli = Cli::try_parse_from(["hatch", "push", "./myapp.zip", "--path", "/release/v1"]).unwrap();
        if let Commands::Push { file, path, force } = cli.command {
            assert_eq!(file, std::path::PathBuf::from("./myapp.zip"));
            assert_eq!(path, "/release/v1");
            assert!(!force);
        } else {
            panic!("expected Push");
        }
    }

    #[test]
    fn parse_push_with_force_flag() {
        let cli = Cli::try_parse_from(["hatch", "push", "./f.zip", "--path", "/r", "--force"]).unwrap();
        if let Commands::Push { force, .. } = cli.command {
            assert!(force);
        }
    }

    #[test]
    fn parse_drop_command_with_yes() {
        let cli = Cli::try_parse_from(["hatch", "drop", "file.zip", "--path", "/release/v1", "--yes"]).unwrap();
        if let Commands::Drop { file, yes, .. } = cli.command {
            assert_eq!(file, "file.zip");
            assert!(yes);
        }
    }

    #[test]
    fn parse_list_with_max_keys() {
        let cli = Cli::try_parse_from(["hatch", "list", "--path", "/release/v1", "--max-keys", "50"]).unwrap();
        if let Commands::List { max_keys, json, .. } = cli.command {
            assert_eq!(max_keys, 50);
            assert!(!json);
        }
    }

    #[test]
    fn list_max_keys_defaults_to_100() {
        let cli = Cli::try_parse_from(["hatch", "list", "--path", "/release/v1"]).unwrap();
        if let Commands::List { max_keys, .. } = cli.command {
            assert_eq!(max_keys, 100);
        }
    }

    #[test]
    fn parse_info_command() {
        let cli = Cli::try_parse_from(["hatch", "info", "file.zip", "--path", "/release/v1"]).unwrap();
        if let Commands::Info { file, path } = cli.command {
            assert_eq!(file, "file.zip");
            assert_eq!(path, "/release/v1");
        }
    }

    #[test]
    fn parse_global_target_flag() {
        let cli = Cli::try_parse_from([
            "hatch", "push", "./f.zip", "--path", "/r", "--target", "https://s3.example.com",
        ]).unwrap();
        assert_eq!(cli.target, Some("https://s3.example.com".to_string()));
    }

    // --- Edge cases ---

    #[test]
    fn max_keys_rejects_zero() {
        let err = Cli::try_parse_from(["hatch", "list", "--path", "/r", "--max-keys", "0"]);
        assert!(err.is_err());
    }

    #[test]
    fn max_keys_rejects_negative() {
        let err = Cli::try_parse_from(["hatch", "list", "--path", "/r", "--max-keys", "-5"]);
        assert!(err.is_err());
    }

    #[test]
    fn max_keys_accepts_1() {
        let cli = Cli::try_parse_from(["hatch", "list", "--path", "/r", "--max-keys", "1"]).unwrap();
        if let Commands::List { max_keys, .. } = cli.command {
            assert_eq!(max_keys, 1);
        }
    }

    #[test]
    fn max_keys_accepts_1000() {
        let cli = Cli::try_parse_from(["hatch", "list", "--path", "/r", "--max-keys", "1000"]).unwrap();
        if let Commands::List { max_keys, .. } = cli.command {
            assert_eq!(max_keys, 1000);
        }
    }

    #[test]
    fn max_keys_rejects_1001() {
        let err = Cli::try_parse_from(["hatch", "list", "--path", "/r", "--max-keys", "1001"]);
        assert!(err.is_err());
    }

    #[test]
    fn push_missing_path_fails() {
        let err = Cli::try_parse_from(["hatch", "push", "file.zip"]);
        assert!(err.is_err());
    }

    #[test]
    fn drop_missing_path_fails() {
        let err = Cli::try_parse_from(["hatch", "drop", "file.zip"]);
        assert!(err.is_err());
    }

    #[test]
    fn list_missing_path_fails() {
        let err = Cli::try_parse_from(["hatch", "list"]);
        assert!(err.is_err());
    }

    #[test]
    fn no_subcommand_fails() {
        let err = Cli::try_parse_from(["hatch"]);
        assert!(err.is_err());
    }

    #[test]
    fn target_without_subcommand_fails() {
        let err = Cli::try_parse_from(["hatch", "--target", "https://x.com"]);
        assert!(err.is_err());
    }

    #[test]
    fn list_json_flag_parsed() {
        let cli = Cli::try_parse_from(["hatch", "list", "--path", "/r", "--json"]).unwrap();
        if let Commands::List { json, .. } = cli.command {
            assert!(json);
        }
    }

    #[test]
    fn drop_defaults_yes_to_false() {
        let cli = Cli::try_parse_from(["hatch", "drop", "f.zip", "--path", "/r"]).unwrap();
        if let Commands::Drop { yes, .. } = cli.command {
            assert!(!yes);
        }
    }
}
