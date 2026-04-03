#!/usr/bin/env sh
# Installs hatch for Linux and macOS.
# For Windows, use: npm install -g @AgoraBuild/hatch
set -e

REPO="Agora-Build/Hatch"
INSTALL_DIR="/usr/local/bin"

detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    case "$OS" in
        linux)
            case "$ARCH" in
                x86_64)  echo "x86_64-unknown-linux-gnu" ;;
                aarch64) echo "aarch64-unknown-linux-gnu" ;;
                *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
            esac ;;
        darwin)
            case "$ARCH" in
                x86_64) echo "x86_64-apple-darwin" ;;
                arm64)  echo "aarch64-apple-darwin" ;;
                *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
            esac ;;
        *) echo "Unsupported OS: $OS" >&2; exit 1 ;;
    esac
}

PLATFORM=$(detect_platform)
LATEST=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"tag_name"' \
    | sed 's/.*"tag_name": *"\(.*\)".*/\1/')

if [ -z "$LATEST" ]; then
    echo "Failed to determine latest release." >&2
    exit 1
fi

URL="https://github.com/$REPO/releases/download/$LATEST/hatch-$PLATFORM"
TMPFILE=$(mktemp)
trap 'rm -f "$TMPFILE"' EXIT

echo "Installing hatch $LATEST ($PLATFORM)..."
curl -fsSL "$URL" -o "$TMPFILE"
chmod +x "$TMPFILE"

if [ -w "$INSTALL_DIR" ]; then
    mv "$TMPFILE" "$INSTALL_DIR/hatch"
else
    sudo mv "$TMPFILE" "$INSTALL_DIR/hatch"
fi
trap - EXIT

echo "Installed: $INSTALL_DIR/hatch"
hatch --version 2>/dev/null || true
