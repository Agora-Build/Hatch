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
  HATCH_PUBLIC_URL=<url>       Public CDN URL (default: https://dl.agora.build)\n\
\n\
  'list' and 'info' work without credentials:\n\
    list  — tries anonymous S3; falls back with a helpful error if bucket is private\n\
    info  — uses HTTP HEAD/GET against HATCH_PUBLIC_URL, no auth required\n\
\n\
SETUP\n\
  1. Create an S3-compatible bucket (e.g. Cloudflare R2, AWS S3)\n\
  2. Create an API token with Object Read & Write permissions\n\
     Copy the Access Key ID → HATCH_ACCESS_KEY\n\
     Copy the Secret Access Key → HATCH_SECRET_KEY\n\
  3. Set HATCH_BUCKET to your bucket name\n\
  4. Optional: set HATCH_PUBLIC_URL to your custom CDN domain\n\
\n\
EXAMPLE .env\n\
  HATCH_ACCESS_KEY=abc123def456\n\
  HATCH_SECRET_KEY=xyz789secret\n\
  HATCH_BUCKET=releases\n\
  HATCH_PUBLIC_URL=https://dl.agora.build\n\
\n\
RELEASE PATH CONVENTION\n\
  /release/<product>/<major_version>/\n\
  e.g. /release/myapp/v1/\n\
\n\
  File name should include full version and build info:\n\
  <name>_v<version>_<build>.zip\n\
  e.g. myapp_v1.0_build42.zip\n\
\n\
  Full URL result:\n\
  https://dl.agora.build/release/myapp/v1/myapp_v1.0_build42.zip\n\
\n\
  Push example:\n\
  hatch push myapp_v1.0_build42.zip --path /release/myapp/v1"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Override the S3-compatible endpoint (overrides HATCH_ENDPOINT and HATCH_PUBLIC_URL)
    #[arg(long, global = true)]
    pub endpoint: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Upload a file to a release path
    Push {
        /// Local file to upload
        file: std::path::PathBuf,
        /// Release path prefix (e.g. /release/myapp/v1)
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
        /// Maximum number of results (default: 100, max: 500)
        #[arg(long, default_value = "100", value_parser = clap::value_parser!(u32).range(1..=500))]
        max_keys: u32,
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
            assert_eq!(max_keys, 50u32);
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
    fn parse_global_endpoint_flag() {
        let cli = Cli::try_parse_from([
            "hatch", "push", "./f.zip", "--path", "/r", "--endpoint", "https://s3.example.com",
        ]).unwrap();
        assert_eq!(cli.endpoint, Some("https://s3.example.com".to_string()));
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
    fn max_keys_accepts_500() {
        let cli = Cli::try_parse_from(["hatch", "list", "--path", "/r", "--max-keys", "500"]).unwrap();
        if let Commands::List { max_keys, .. } = cli.command {
            assert_eq!(max_keys, 500);
        }
    }

    #[test]
    fn max_keys_rejects_501() {
        let err = Cli::try_parse_from(["hatch", "list", "--path", "/r", "--max-keys", "501"]);
        assert!(err.is_err());
    }

    #[test]
    fn max_keys_rejects_1000() {
        let err = Cli::try_parse_from(["hatch", "list", "--path", "/r", "--max-keys", "1000"]);
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
    fn endpoint_without_subcommand_fails() {
        let err = Cli::try_parse_from(["hatch", "--endpoint", "https://x.com"]);
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
