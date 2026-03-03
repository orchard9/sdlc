#!/bin/sh
# Install ponder — feature lifecycle CLI
# Binary: ponder  |  Alias: sdlc -> ponder
#
# Usage:
#   curl -sSfL https://raw.githubusercontent.com/orchard9/sdlc/main/install.sh | sh
#
# Override install directory:
#   PONDER_INSTALL=/usr/local/bin curl -sSfL ... | sh

set -e

REPO="orchard9/sdlc"
INSTALL_DIR="${PONDER_INSTALL:-$HOME/.local/bin}"

# ── Detect OS and architecture ──────────────────────────────────────────────

OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Darwin)
    case "$ARCH" in
      arm64)  TARGET="aarch64-apple-darwin" ;;
      x86_64) TARGET="x86_64-apple-darwin" ;;
      *) echo "error: unsupported macOS architecture: $ARCH" >&2 && exit 1 ;;
    esac
    ;;
  Linux)
    case "$ARCH" in
      x86_64)         TARGET="x86_64-unknown-linux-musl" ;;
      aarch64 | arm64) TARGET="aarch64-unknown-linux-musl" ;;
      *) echo "error: unsupported Linux architecture: $ARCH" >&2 && exit 1 ;;
    esac
    ;;
  *)
    echo "error: unsupported OS: $OS" >&2
    echo "For Windows, use install.ps1:" >&2
    echo "  irm https://raw.githubusercontent.com/orchard9/sdlc/main/install.ps1 | iex" >&2
    exit 1
    ;;
esac

# ── Resolve latest version ───────────────────────────────────────────────────

if command -v curl >/dev/null 2>&1; then
  FETCH="curl -sSfL"
elif command -v wget >/dev/null 2>&1; then
  FETCH="wget -qO-"
else
  echo "error: curl or wget is required" >&2
  exit 1
fi

VERSION=$($FETCH "https://api.github.com/repos/$REPO/releases/latest" \
  | grep '"tag_name"' \
  | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

if [ -z "$VERSION" ]; then
  echo "error: could not determine latest version" >&2
  exit 1
fi

# ── Download and install ─────────────────────────────────────────────────────

URL="https://github.com/$REPO/releases/download/$VERSION/ponder-$TARGET.tar.gz"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

printf "Installing ponder %s (%s)...\n" "$VERSION" "$TARGET"

$FETCH "$URL" | tar xz -C "$TMP"

mkdir -p "$INSTALL_DIR"
mv "$TMP/ponder" "$INSTALL_DIR/ponder"
chmod +x "$INSTALL_DIR/ponder"

# Create sdlc alias
ln -sf "$INSTALL_DIR/ponder" "$INSTALL_DIR/sdlc"

printf "  \033[32m✓\033[0m ponder -> %s/ponder\n" "$INSTALL_DIR"
printf "  \033[32m✓\033[0m sdlc   -> %s/ponder  (alias)\n" "$INSTALL_DIR"

# ── PATH hint ────────────────────────────────────────────────────────────────

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    printf "\n\033[33m!\033[0m %s is not in your PATH.\n" "$INSTALL_DIR"
    printf "  Add this to your shell profile (~/.bashrc, ~/.zshrc, ~/.profile):\n"
    printf "    export PATH=\"\$HOME/.local/bin:\$PATH\"\n\n"
    ;;
esac

printf "\nRun \033[1mponder --version\033[0m or \033[1msdlc --version\033[0m to verify.\n"
