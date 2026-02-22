#!/bin/bash
set -e
set -o pipefail

# MATO Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/mr-kelly/mato/main/install.sh | bash

# Override with: MATO_REPO=owner/repo
REPO="${MATO_REPO:-mr-kelly/mato}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        OS_TYPE="linux"
        ;;
    Darwin*)
        OS_TYPE="macos"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH_TYPE="x86_64"
        ;;
    aarch64|arm64)
        ARCH_TYPE="aarch64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

BINARY_NAME="mato-${OS_TYPE}-${ARCH_TYPE}"

echo "Installing MATO for ${OS_TYPE}-${ARCH_TYPE}..."

# Get latest release metadata
RELEASE_JSON=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")
LATEST_RELEASE=$(printf '%s' "$RELEASE_JSON" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_RELEASE" ]; then
    echo "Failed to get latest release for ${REPO}"
    exit 1
fi

echo "Latest version: $LATEST_RELEASE"

# Resolve download URL from release assets.
# Supports both:
#   mato-<os>-<arch>.tar.gz
#   mato-<os>-<arch>-<tag>.tar.gz
DOWNLOAD_URL=$(
    printf '%s' "$RELEASE_JSON" \
    | grep '"browser_download_url":' \
    | sed -E 's/.*"([^"]+)".*/\1/' \
    | grep -E "/${BINARY_NAME}(-[^/]+)?\\.tar\\.gz$" \
    | head -n1
)

if [ -z "$DOWNLOAD_URL" ]; then
    echo "No matching binary asset found for ${BINARY_NAME} in release ${LATEST_RELEASE}"
    echo "Available assets:"
    printf '%s' "$RELEASE_JSON" \
      | grep '"name":' \
      | sed -E 's/.*"name": "([^"]+)".*/  - \1/' \
      || true
    exit 1
fi

echo "Downloading from: $DOWNLOAD_URL"

# Create temp directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

# Download and extract
cd "$TMP_DIR"
curl -fsSL "$DOWNLOAD_URL" -o mato.tar.gz
tar xzf mato.tar.gz

# Install
mkdir -p "$INSTALL_DIR"
mv mato "$INSTALL_DIR/mato"
chmod +x "$INSTALL_DIR/mato"

echo ""
echo "✅ MATO installed successfully to $INSTALL_DIR/mato"
echo ""

# Check if in PATH
if echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo "You can now run: mato"
else
    echo "⚠️  Add $INSTALL_DIR to your PATH:"
    echo ""
    echo "    echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.bashrc"
    echo "    source ~/.bashrc"
    echo ""
    echo "Or run directly: $INSTALL_DIR/mato"
fi

echo ""
echo "📚 Documentation: https://github.com/${REPO}"
echo "🐛 Report issues: https://github.com/${REPO}/issues"
