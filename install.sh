#!/usr/bin/env bash
set -e

# Sloth installer script for Linux and macOS
REPO="syedzahidsaleem/sloth-tui"
INSTALL_DIR="/usr/local/bin"

echo "🦥 Installing Sloth..."

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
    linux)
        case "$ARCH" in
            x86_64|amd64)
                TARGET="linux-x86_64"
                ;;
            aarch64|arm64)
                TARGET="linux-aarch64"
                ;;
            *)
                echo "Unsupported architecture: $ARCH on Linux"
                exit 1
                ;;
        esac
        ;;
    darwin)
        case "$ARCH" in
            x86_64|amd64)
                TARGET="macos-x86_64"
                ;;
            aarch64|arm64)
                TARGET="macos-aarch64"
                ;;
            *)
                echo "Unsupported architecture: $ARCH on macOS"
                exit 1
                ;;
        esac
        ;;
    *)
        echo "Unsupported operating system: $OS"
        echo "On Windows, please install via: cargo install sloth-tui or winget install syedzahidsaleem.sloth-tui"
        exit 1
        ;;
esac

ARCHIVE_NAME="sloth-tui-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ARCHIVE_NAME}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading ${ARCHIVE_NAME} from ${DOWNLOAD_URL}..."
if curl -fsSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${ARCHIVE_NAME}"; then
    tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "$TMP_DIR"
    
    BIN_SRC="${TMP_DIR}/sloth-tui"
    if [ ! -f "$BIN_SRC" ]; then
        echo "Binary not found in archive!"
        exit 1
    fi
    
    chmod +x "$BIN_SRC"
    
    # Determine destination directory
    if [ -w "$INSTALL_DIR" ]; then
        DEST="$INSTALL_DIR/sloth-tui"
        mv "$BIN_SRC" "$DEST"
    elif command -v sudo >/dev/null 2>&1; then
        echo "Installing to ${INSTALL_DIR} (requires sudo permissions)..."
        sudo mv "$BIN_SRC" "$INSTALL_DIR/sloth-tui"
    else
        USER_BIN="$HOME/.local/bin"
        mkdir -p "$USER_BIN"
        mv "$BIN_SRC" "$USER_BIN/sloth-tui"
        echo "Installed to $USER_BIN/sloth-tui. Ensure $USER_BIN is in your PATH."
    fi
else
    echo "Pre-built binary download failed or release not yet published."
    echo "Falling back to 'cargo install sloth-tui'..."
    if command -v cargo >/dev/null 2>&1; then
        cargo install sloth-tui
    else
        echo "Error: cargo is not installed. Please install Rust or download prebuilt binaries from:"
        echo "https://github.com/${REPO}/releases"
        exit 1
    fi
fi

# Check for mpv prerequisite
if ! command -v mpv >/dev/null 2>&1; then
    echo ""
    echo "⚠️ Notice: 'mpv' was not detected in your PATH."
    echo "Sloth requires mpv for streaming playback."
    echo "  - Ubuntu/Debian: sudo apt install mpv"
    echo "  - Arch Linux:    sudo pacman -S mpv"
    echo "  - macOS:         brew install mpv"
fi

echo ""
echo "🎉 Sloth installed successfully! Run 'sloth-tui' to start streaming."
