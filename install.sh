#!/usr/bin/env bash
set -euo pipefail

# infra-import installer
# Usage: curl -fsSL https://raw.githubusercontent.com/super-kawaii-projects/infra-import/main/install.sh | bash

REPO="super-kawaii-projects/infra-import"
INSTALL_DIR="/usr/local/bin"
BINARY="infra-import"

# Detect OS and architecture
detect_platform() {
    local os arch

    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m)"

    case "$os" in
        linux)  os="linux" ;;
        darwin) os="darwin" ;;
        *)      echo "Unsupported OS: $os" >&2; exit 1 ;;
    esac

    case "$arch" in
        x86_64|amd64)  arch="amd64" ;;
        aarch64|arm64) arch="arm64" ;;
        *)             echo "Unsupported architecture: $arch" >&2; exit 1 ;;
    esac

    echo "${os}-${arch}"
}

# Get latest release tag from GitHub
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | cut -d'"' -f4
}

main() {
    echo "Installing infra-import..."
    echo ""

    local platform version url tmp

    platform="$(detect_platform)"
    version="$(get_latest_version)"

    if [ -z "$version" ]; then
        echo "Error: Could not determine latest version." >&2
        echo "Check that the repository exists: https://github.com/${REPO}" >&2
        exit 1
    fi

    echo "  Version:  $version"
    echo "  Platform: $platform"
    echo ""

    url="https://github.com/${REPO}/releases/download/${version}/${BINARY}-${platform}"

    tmp="$(mktemp)"
    echo "Downloading ${url}..."
    curl -fsSL -o "$tmp" "$url"
    chmod +x "$tmp"

    # Install - try without sudo first
    if [ -w "$INSTALL_DIR" ]; then
        mv "$tmp" "${INSTALL_DIR}/${BINARY}"
    else
        echo "Need sudo to install to ${INSTALL_DIR}"
        sudo mv "$tmp" "${INSTALL_DIR}/${BINARY}"
    fi

    echo ""
    echo "✓ Installed ${BINARY} ${version} to ${INSTALL_DIR}/${BINARY}"
    echo ""
    echo "Prerequisites:"
    echo "  - AWS CLI installed and configured (aws configure)"
    echo ""
    echo "Usage:"
    echo "  infra-import --region us-east-1"
    echo "  infra-import --region us-east-1 --profile production"
    echo "  infra-import --scope kubernetes --region us-west-2"
    echo ""
}

main
