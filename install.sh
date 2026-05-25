#!/usr/bin/env bash
#
# Install hatch — secure, versioned file release CLI for S3-compatible storage.
#
# Quick install:
#   curl -fsSL https://dl.agora.build/hatch/install.sh | bash
#
# Options (via env vars):
#   HATCH_VERSION=0.1.0     Pin a specific version (default: latest)
#   HATCH_BASE_URL=...      Override download base URL
#   HATCH_INSTALL_DIR=...   Override install directory
#
# For Windows, use: npm install -g @agora-build/hatch
#
set -euo pipefail

BASE_URL="${HATCH_BASE_URL:-https://dl.agora.build/hatch/releases}"

# ── Helpers ──────────────────────────────────────────────────────────

die()  { echo "error: $*" >&2; exit 1; }
info() { echo "  $*"; }

# ── Detect platform ──────────────────────────────────────────────────

detect_platform() {
  local os arch
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"

  case "$os" in
    linux)  os="linux" ;;
    darwin) os="darwin" ;;
    *)      die "Unsupported OS: $os (supported: linux, darwin)" ;;
  esac

  case "$arch" in
    x86_64|amd64)  arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *)             die "Unsupported architecture: $arch (supported: x86_64, aarch64)" ;;
  esac

  echo "${os}-${arch}"
}

# ── Resolve version ─────────────────────────────────────────────────

resolve_version() {
  if [ -n "${HATCH_VERSION:-}" ]; then
    echo "$HATCH_VERSION"
    return
  fi

  local url="${BASE_URL}/latest"
  local version
  version="$(curl -fsSL "$url" 2>/dev/null)" \
    || die "Failed to fetch latest version from $url"
  version="$(echo "$version" | tr -d '[:space:]')"
  [ -n "$version" ] || die "Empty version returned from $url"
  echo "$version"
}

# ── Pick install directory ───────────────────────────────────────────

pick_install_dir() {
  if [ -n "${HATCH_INSTALL_DIR:-}" ]; then
    mkdir -p "$HATCH_INSTALL_DIR"
    echo "$HATCH_INSTALL_DIR"
    return
  fi

  if [ -w "/usr/local/bin" ]; then
    echo "/usr/local/bin"
  else
    local dir="${HOME}/.local/bin"
    mkdir -p "$dir"
    echo "$dir"
  fi
}

# ── Main ─────────────────────────────────────────────────────────────

main() {
  echo "Installing hatch..."

  local platform version install_dir
  platform="$(detect_platform)"
  version="$(resolve_version)"
  install_dir="$(pick_install_dir)"

  local archive="hatch-v${version}-${platform}.tar.gz"
  local url="${BASE_URL}/v${version}/${archive}"

  info "Version:  ${version}"
  info "Platform: ${platform}"
  info "From:     ${url}"
  info "To:       ${install_dir}/hatch"

  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  curl -fSL --progress-bar "$url" -o "${tmpdir}/${archive}" \
    || die "Download failed: ${url}"

  tar -xzf "${tmpdir}/${archive}" -C "$tmpdir" \
    || die "Failed to extract ${archive}"

  [ -f "${tmpdir}/hatch" ] \
    || die "Tarball is missing hatch binary; corrupted release?"

  chmod +x "${tmpdir}/hatch"

  if [ -w "$install_dir" ]; then
    mv "${tmpdir}/hatch" "${install_dir}/hatch"
  else
    sudo mv "${tmpdir}/hatch" "${install_dir}/hatch"
  fi

  echo ""
  if command -v hatch >/dev/null 2>&1; then
    echo "Installed: $(hatch --version 2>/dev/null || echo hatch)"
  else
    echo "Installed to ${install_dir}/hatch"
    case ":${PATH}:" in
      *":${install_dir}:"*) ;;
      *)
        echo ""
        echo "Add to your PATH:"
        echo "  export PATH=\"${install_dir}:\$PATH\""
        ;;
    esac
  fi
}

main "$@"
