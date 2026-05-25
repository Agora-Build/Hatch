# @agora-build/hatch

Secure, versioned file release CLI for S3-compatible storage.

Hatch uploads, manages, and distributes files in versioned release paths on S3-compatible storage (Cloudflare R2, AWS S3, MinIO, etc.), with automatic checksum generation.

## Install

```bash
npm install -g @agora-build/hatch
```

Or via shell script:

```bash
curl -fsSL https://dl.agora.build/hatch/install.sh | bash
```

Both download a prebuilt binary for your platform (linux-x64, linux-arm64, darwin-x64, darwin-arm64, win32-x64).

## Quick Start

```bash
# Configure credentials (.env or environment variables)
export HATCH_ACCESS_KEY=your_access_key
export HATCH_SECRET_KEY=your_secret_key
export HATCH_BUCKET=releases

# Upload a file
hatch push myapp_v1.0_build42.zip --path /release/myapp/v1
# => https://dl.agora.build/release/myapp/v1/myapp_v1.0_build42.zip

# List files
hatch list --path /release/myapp/v1

# Inspect metadata and checksums
hatch info myapp_v1.0_build42.zip --path /release/myapp/v1

# Delete a file
hatch drop myapp_v1.0_build42.zip --path /release/myapp/v1
```

## Commands

```bash
hatch push <file> --path <path>           # Upload (auto-generates .md5 and .sha256 sidecars)
hatch push <file> --path <path> --force   # Overwrite if exists
hatch list --path <path>                  # List files
hatch list --path <path> --json           # List as JSON
hatch info <file> --path <path>           # Show metadata and checksums
hatch drop <file> --path <path>           # Delete a single file
hatch drop --path <path> --yes            # Batch delete everything under path
hatch drop --path <path> --filter <regex> # Batch delete with regex filter
hatch drop --path <path> --dry-run        # Preview what would be deleted
```

## Configuration

```bash
HATCH_ACCESS_KEY=<key>       # Required for: push, drop
HATCH_SECRET_KEY=<secret>    # Required for: push, drop
HATCH_BUCKET=<bucket>        # Required for: push, drop
HATCH_PUBLIC_URL=<url>       # Public CDN URL (default: https://dl.agora.build)
```

Config priority: env vars > `--env-file` > local `.env` > `~/.config/hatch/.env`

```bash
hatch list --path /release --env-file ~/.config/hatch/artifacts.env
```

## Documentation

Full documentation and source: [github.com/Agora-Build/Hatch](https://github.com/Agora-Build/Hatch)

## License

MIT
