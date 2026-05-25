# Hatch

Secure, versioned file release CLI for S3-compatible storage.

Hatch uploads, manages, and distributes files in versioned release paths on S3-compatible storage (Cloudflare R2, AWS S3, MinIO, etc.), with automatic checksum generation.

## Install

```bash
curl -fsSL https://artifacts.agora.build/hatch/install.sh | bash
```

Or via npm:

```bash
npm install -g @agora-build/hatch
```

Both download a prebuilt binary for your platform (linux-x64, linux-arm64, darwin-x64, darwin-arm64, win32-x64).

From source:

```bash
cargo install --git https://github.com/Agora-Build/Hatch
```

## Quick Start

```bash
# 1. Configure credentials
cat > .env <<EOF
HATCH_ACCESS_KEY=your_access_key
HATCH_SECRET_KEY=your_secret_key
HATCH_BUCKET=releases
EOF

# 2. Upload a file
hatch push myapp_v1.0_build42.zip --path /release/myapp/v1
# => https://artifacts.agora.build/release/myapp/v1/myapp_v1.0_build42.zip

# 3. List files at a path
hatch list --path /release/myapp/v1

# 4. Inspect a file (metadata + checksums)
hatch info myapp_v1.0_build42.zip --path /release/myapp/v1

# 5. Delete a file
hatch drop myapp_v1.0_build42.zip --path /release/myapp/v1

# 6. Batch delete (e.g. clean up old jobs)
hatch drop --path /jobs/13125 --yes                          # everything under jobs/13125/
hatch drop --path /jobs --filter "^jobs/131" --yes           # regex filter
hatch drop --path /jobs --filter "^jobs/131" --dry-run       # preview first
```

## Commands

```bash
hatch push <file> --path <path>           # Upload a file (auto-generates .md5 and .sha256 sidecars)
hatch push <file> --path <path> --force   # Overwrite if exists
hatch list --path <path>                  # List files at a release path
hatch list --path <path> --json           # List as JSON
hatch list --path <path> --max-keys 50    # Limit results (max 500)
hatch info <file> --path <path>           # Show metadata, size, and checksums
hatch drop <file> --path <path>           # Delete a single file
hatch drop <file> --path <path> --yes     # Skip confirmation (for CI)
hatch drop --path <path> --yes            # Batch delete everything under path
hatch drop --path <path> --filter <regex> # Batch delete with regex filter on keys
hatch drop --path <path> --dry-run        # Preview what would be deleted
```

`push` and `drop` require credentials. `list` and `info` work without credentials on public buckets.

## Configuration

Credentials are loaded in this order (highest priority first):

1. **Environment variables** — always win
2. **`--env-file <path>`** or **`HATCH_ENV_FILE`** — explicit file
3. **Local `.env`** — in working directory (skipped if `--env-file` is set)
4. **`~/.config/hatch/.env`** — global defaults

```bash
HATCH_ACCESS_KEY=<key>       # Required for: push, drop
HATCH_SECRET_KEY=<secret>    # Required for: push, drop
HATCH_BUCKET=<bucket>        # Required for: push, drop
HATCH_PUBLIC_URL=<url>       # Public CDN URL (default: https://artifacts.agora.build)
```

You can keep shared credentials in `~/.config/hatch/.env` and override per-project with a local `.env`, or use `--env-file` to point at a specific config:

```bash
hatch list --path /release --env-file ~/.config/hatch/artifacts.env
```

## Setup

1. Create an S3-compatible bucket (e.g. Cloudflare R2, AWS S3)
2. Create an API token with Object Read & Write permissions
   - **Cloudflare R2:** R2 Object Storage → Manage R2 API Tokens → Create API Token
   - **AWS S3:** IAM → Create access key
3. Copy the Access Key ID and Secret Access Key into your `.env`
4. Set `HATCH_BUCKET` to your bucket name
5. Optionally set `HATCH_PUBLIC_URL` to your custom CDN domain

## Release Path Convention

```
/release/<product>/<major_version>/
```

File names should include full version and build info:

```
hatch push myapp_v1.0_build42.zip --path /release/myapp/v1
# => https://artifacts.agora.build/release/myapp/v1/myapp_v1.0_build42.zip
```

## Features

- Automatic MD5 and SHA256 checksum sidecar generation on push
- Overwrite protection (`--force` to override)
- Safe delete with confirmation prompt (`--yes` for CI)
- Batch delete by prefix with optional regex filtering (`--filter`)
- Dry run mode to preview batch operations (`--dry-run`)
- JSON output for `list` (`--json`)
- Truncation warning when results exceed `--max-keys`
- Anonymous access for `list` and `info` on public buckets
- URL-encoded filenames in output URLs
- Works with any S3-compatible storage

## License

MIT
