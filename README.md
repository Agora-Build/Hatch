# Hatch

Secure, versioned file release CLI for S3-compatible storage.

Hatch uploads, manages, and distributes files in versioned release paths on S3-compatible storage (Cloudflare R2, AWS S3, MinIO, etc.), with automatic checksum generation.

## Install

**Shell (Linux/macOS):**

```sh
curl -fsSL https://raw.githubusercontent.com/Agora-Build/Hatch/master/install.sh | sh
```

**npm:**

```sh
npm install -g @agora-build/hatch
```

**From source:**

```sh
cargo install --git https://github.com/Agora-Build/Hatch
```

## Quick Start

```sh
# 1. Create a .env file with your credentials
cat > .env <<EOF
HATCH_ACCESS_KEY=your_access_key
HATCH_SECRET_KEY=your_secret_key
HATCH_BUCKET=releases
EOF

# 2. Upload a file
hatch push myapp_v1.0_build42.zip --path /release/myapp/v1
# => https://dl.agora.build/release/myapp/v1/myapp_v1.0_build42.zip

# 3. List files
hatch list --path /release/myapp/v1

# 4. Check file metadata and checksums
hatch info myapp_v1.0_build42.zip --path /release/myapp/v1

# 5. Delete a file
hatch drop myapp_v1.0_build42.zip --path /release/myapp/v1
```

## Commands

| Command | Description |
|---------|-------------|
| `push`  | Upload a file to a release path |
| `drop`  | Delete a file (requires confirmation) |
| `list`  | List files at a release path |
| `info`  | Show metadata and checksums for a file |

`push` and `drop` require credentials. `list` and `info` work without credentials against public buckets.

## Configuration

Add to a `.env` file in your working directory, or export as environment variables:

```sh
HATCH_ACCESS_KEY=<key>       # Required for: push, drop
HATCH_SECRET_KEY=<secret>    # Required for: push, drop
HATCH_BUCKET=<bucket>        # Required for: push, drop
HATCH_PUBLIC_URL=<url>       # Public CDN URL (default: https://dl.agora.build)
```

## Setup

1. Create an S3-compatible bucket (e.g. Cloudflare R2, AWS S3)
2. Create an API token with Object Read & Write permissions
3. Copy the Access Key ID and Secret Access Key into your `.env`
4. Set `HATCH_BUCKET` to your bucket name
5. Optionally set `HATCH_PUBLIC_URL` to your custom CDN domain

## Release Path Convention

```
/release/<product>/<major_version>/
```

File names should include full version and build info:

```
<name>_v<version>_<build>.zip
```

Example:

```
hatch push myapp_v1.0_build42.zip --path /release/myapp/v1
# => https://dl.agora.build/release/myapp/v1/myapp_v1.0_build42.zip
```

## Features

- Automatic MD5 and SHA256 checksum sidecar generation on push
- Overwrite protection (`--force` to override)
- Safe delete with confirmation prompt (`--yes` for CI)
- JSON output for `list` (`--json`)
- Anonymous access for `list` and `info` on public buckets
- Works with any S3-compatible storage

## License

MIT
