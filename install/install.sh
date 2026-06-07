#!/bin/bash
set -e

REPO="subhobhai943/sub-code"
BIN_DIR="$HOME/.local/bin"

echo "Installing SUB CODE..."

# Detect OS
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "$OS" in
    linux*)     OS_NAME="linux" ;;
    darwin*)    OS_NAME="macos" ;;
    *)          echo "Unsupported OS: $OS"; exit 1 ;;
esac

# Detect Architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64) 
        ARCH_NAME="x86_64" 
        ;;
    aarch64|arm64) 
        if [ "$OS_NAME" = "macos" ]; then
            ARCH_NAME="arm64"
        else
            ARCH_NAME="aarch64"
        fi
        ;;
    *) 
        echo "Unsupported architecture: $ARCH"; exit 1 
        ;;
esac

BINARY_NAME="subcode-${OS_NAME}-${ARCH_NAME}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${BINARY_NAME}"
CHECKSUM_URL="${DOWNLOAD_URL}.sha256"

# Create bin directory if it doesn't exist
mkdir -p "$BIN_DIR"

echo "Downloading ${BINARY_NAME}..."
curl -sSL "$DOWNLOAD_URL" -o "$BIN_DIR/subcode"
curl -sSL "$CHECKSUM_URL" -o "$BIN_DIR/subcode.sha256"

echo "Verifying checksum..."
cd "$BIN_DIR"

if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL_HASH=$(cat subcode.sha256 | awk '{print $1}')
    echo "$ACTUAL_HASH  subcode" | sha256sum -c -
elif command -v shasum >/dev/null 2>&1; then
    ACTUAL_HASH=$(cat subcode.sha256 | awk '{print $1}')
    echo "$ACTUAL_HASH  subcode" | shasum -a 256 -c -
else
    echo "Warning: Could not find sha256sum or shasum to verify the binary."
fi

rm -f subcode.sha256
chmod +x "$BIN_DIR/subcode"
cd - >/dev/null

echo "Adding $BIN_DIR to PATH if needed..."

# Add to Bash
if [ -f "$HOME/.bashrc" ]; then
    if ! grep -q "$BIN_DIR" "$HOME/.bashrc"; then
        echo 'export PATH="'"$BIN_DIR"':$PATH"' >> "$HOME/.bashrc"
        echo "Added to ~/.bashrc"
    fi
fi

# Add to Zsh (Termux and macOS default)
if [ -f "$HOME/.zshrc" ]; then
    if ! grep -q "$BIN_DIR" "$HOME/.zshrc"; then
        echo 'export PATH="'"$BIN_DIR"':$PATH"' >> "$HOME/.zshrc"
        echo "Added to ~/.zshrc"
    fi
fi

# Add to Fish
FISH_CONFIG="$HOME/.config/fish/config.fish"
if [ -d "$HOME/.config/fish" ]; then
    if [ ! -f "$FISH_CONFIG" ] || ! grep -q "$BIN_DIR" "$FISH_CONFIG"; then
        echo "fish_add_path \"$BIN_DIR\"" >> "$FISH_CONFIG"
        echo "Added to ~/.config/fish/config.fish"
    fi
fi

export PATH="$BIN_DIR:$PATH"

echo "Running SUB CODE setup wizard..."
subcode --setup

echo "Installation complete! Please restart your terminal or source your shell config."
