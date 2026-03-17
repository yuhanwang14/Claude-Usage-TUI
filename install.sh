#!/bin/sh
# install.sh — Install claude-usage-tui from GitHub Releases
# Usage: curl -fsSL https://raw.githubusercontent.com/yuhanwang14/Claude-Usage-TUI/main/install.sh | sh

set -e

REPO="yuhanwang14/Claude-Usage-TUI"
BINARY="claude-usage-tui"

# Detect OS
OS="$(uname -s)"
case "$OS" in
  Darwin) OS_NAME="apple-darwin" ;;
  Linux)  OS_NAME="unknown-linux-gnu" ;;
  *)
    echo "Unsupported OS: $OS" >&2
    exit 1
    ;;
esac

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
  arm64 | aarch64) ARCH_NAME="aarch64" ;;
  x86_64 | amd64)  ARCH_NAME="x86_64" ;;
  *)
    echo "Unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

TARGET="${ARCH_NAME}-${OS_NAME}"
TARBALL="${BINARY}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${TARBALL}"

echo "Detected: ${OS} / ${ARCH} (target: ${TARGET})"
echo "Downloading ${DOWNLOAD_URL} ..."

# Pick install directory
if [ -w "/usr/local/bin" ]; then
  INSTALL_DIR="/usr/local/bin"
elif command -v sudo >/dev/null 2>&1 && [ "$EUID" != "0" ]; then
  # Offer sudo install if /usr/local/bin exists but isn't writable
  if [ -d "/usr/local/bin" ]; then
    USE_SUDO=1
    INSTALL_DIR="/usr/local/bin"
  else
    INSTALL_DIR="${HOME}/.local/bin"
  fi
else
  INSTALL_DIR="${HOME}/.local/bin"
fi

mkdir -p "$INSTALL_DIR"

# Download and extract
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${TARBALL}"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "${TMP_DIR}/${TARBALL}" "$DOWNLOAD_URL"
else
  echo "Error: neither curl nor wget found." >&2
  exit 1
fi

tar -xzf "${TMP_DIR}/${TARBALL}" -C "$TMP_DIR"

if [ "${USE_SUDO:-0}" = "1" ]; then
  echo "Installing to ${INSTALL_DIR} (requires sudo) ..."
  sudo install -m 0755 "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
else
  install -m 0755 "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
fi

echo ""
echo "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"

# Remind user to add to PATH if needed
case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo ""
    echo "Note: ${INSTALL_DIR} is not in your PATH."
    echo "Add this to your shell profile:"
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    ;;
esac
