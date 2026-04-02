# Hatch — Design Spec

**Date:** 2026-04-02  
**Repo:** github.com/Agora-Build/Hatch  
**Language:** Rust  
**Status:** Approved

---

## Overview

Hatch is a secure, versioned file management and release CLI tool for uploading, managing, and distributing files across S3-compatible cloud storage. It targets CI/CD pipelines and developer workflows, with automatic checksum generation, safe-by-default delete, and multi-platform distribution.

---

## Architecture

Single Rust binary, one Cargo crate (monolithic). All logic lives in focused internal modules.

```
hatch/
├── src/
│   ├── main.rs            # CLI entry: parses args, dispatches to commands
│   ├── cli.rs             # clap-based argument definitions
│   ├── commands/
│   │   ├── push.rs        # upload + checksum generation
│   │   ├── drop.rs        # delete with confirmation
│   │   ├── list.rs        # list with --max-keys cap
│   │   └── info.rs        # show file metadata + checksums
│   ├── storage/
│   │   └── s3.rs          # S3-compatible client (aws-sdk-s3)
│   ├── checksum.rs        # MD5 + SHA256 generation
│   └── credentials.rs     # .env loading + validation
├── npm/
│   ├── hatch/             # main @agora-build/hatch package
│   └── platforms/         # @agora-build/hatch-<platform> packages
├── .github/workflows/
│   └── release.yml        # cross-compile + publish to GH Releases + npm
├── install.sh             # shell install script
└── Cargo.toml
```

**Key dependencies:**
- `clap` (v4, derive) — CLI parsing
- `aws-sdk-s3` — S3-compatible storage client
- `tokio` — async runtime
- `md-5` + `sha2` — checksum computation
- `dotenvy` — `.env` loading
- `indicatif` — upload progress bar
- `anyhow` — error handling

---

## Commands

### `hatch push <file> --path <path> [--force] [--target <endpoint>]`

- Uploads `<file>` to `<path>/<filename>` on the storage target
- Computes MD5 and SHA256 locally before upload; uploads `<file>.md5` and `<file>.sha256` sidecars
- Fails if the file already exists unless `--force` is passed
- Prints a progress bar during upload
- On success, prints the full public URL using the active endpoint: `https://<endpoint>/<path>/<filename>` (default: `https://dl.agora.build/...`)
- `--target <endpoint>` overrides the default `dl.agora.build` endpoint; the printed URL reflects the override

### `hatch drop <file> --path <path> [--yes] [--target <endpoint>]`

- Prompts for confirmation showing the full key before deleting
- `--yes` bypasses the prompt for CI use
- If non-interactive and `--yes` is not passed, exits with a non-zero error
- Also deletes the associated `.md5` and `.sha256` sidecar files
- Fails with a clear error if the file does not exist

### `hatch list --path <path> [--max-keys N] [--json] [--target <endpoint>]`

- Lists objects under a path prefix: name, size, last modified
- Default `--max-keys 100`; maximum allowed value is 1000
- `--json` outputs machine-readable JSON for scripting

### `hatch info <file> --path <path> [--target <endpoint>]`

- Shows metadata for a specific file: size, last modified, ETag
- Fetches and displays stored MD5 and SHA256 checksums from sidecar files if present

---

## Checksum Generation

Every `hatch push` produces two sidecar files:

- `<filename>.md5` — hex MD5 digest
- `<filename>.sha256` — hex SHA256 digest

Both are computed locally by streaming the file in chunks before upload. Sidecar format matches `md5sum`/`sha256sum` standard output:

```
d8e8fca2dc0f896fd7cb4cb0031ba249  myapp.zip
```

Users verify with: `sha256sum -c myapp.zip.sha256`

If a sidecar upload fails after the main file is uploaded, Hatch emits a warning with the full URL and exits with a non-zero code. The main file is not deleted.

---

## Credentials

Credentials are read from `.env` or environment. Only `push` and `drop` require full S3 credentials.

| Variable | Required for | Default |
|----------|-------------|---------|
| `HATCH_ACCESS_KEY` | push, drop | — |
| `HATCH_SECRET_KEY` | push, drop | — |
| `HATCH_BUCKET` | push, drop, list (with credentials) | — |
| `HATCH_ENDPOINT` | optional for all | `https://dl.agora.build` |
| `HATCH_PUBLIC_URL` | optional | same as `HATCH_ENDPOINT` |

- `push` and `drop` fail immediately with a clear error if any required credential is missing
- Credentials are never printed, logged, or included in error output

### list — anonymous mode

`list` does not require credentials. It attempts an anonymous S3 `ListObjectsV2` request against the endpoint. If the storage provider rejects anonymous listing (HTTP 403), Hatch prints a helpful message: `Listing requires credentials — set HATCH_ACCESS_KEY, HATCH_SECRET_KEY, and HATCH_BUCKET`.

If credentials are present, `list` uses them (authenticated mode).

### info — HTTP mode

`info` does not use the S3 API at all. It issues HTTP `HEAD` and `GET` requests to `HATCH_PUBLIC_URL` to retrieve file metadata and sidecar checksums. No credentials needed.

---

## Error Handling

- All S3 errors mapped to human-readable messages
- File-already-exists on push: `Error: file already exists at <path> — use --force to overwrite`
- Network errors include the attempted endpoint
- All failures exit with non-zero status (CI-friendly)
- `--force` existence check happens before upload, not after

---

## Default Storage Target

`dl.agora.build` backed by Cloudflare R2. All commands accept `--target <s3-endpoint>` to override with any S3-compatible endpoint.

---

## Distribution

### GitHub Releases

Triggered on `v*` tags. Cross-compiles for:
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

Each binary is attached to the GitHub Release.

### Shell Install Script (`install.sh`)

Detects OS and architecture, downloads the correct binary from GitHub Releases, installs to `/usr/local/bin/hatch`.

```sh
curl -fsSL https://raw.githubusercontent.com/Agora-Build/Hatch/main/install.sh | sh
```

### npm (`@agora-build/hatch`)

Follows the esbuild/biome platform-package pattern:

- `@agora-build/hatch` — main package with `bin: { hatch: "bin/hatch" }` and `optionalDependencies` pointing to platform packages
- Platform packages (`@agora-build/hatch-linux-x64`, `@agora-build/hatch-darwin-arm64`, etc.) — each contains only the platform binary
- The main package's `bin/hatch` entry point resolves and executes the correct platform binary at runtime

```sh
npm install -g @agora-build/hatch
```

The release workflow publishes all npm packages automatically after attaching binaries to the GitHub Release.
