#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

cd "$PROJECT_ROOT"

log_info "Installing build dependencies..."
if command -v apt-get &> /dev/null; then
    sudo apt-get update -qq
    sudo apt-get install -y \
        debhelper \
        dpkg-dev \
        build-essential \
        libssl-dev \
        pkg-config \
        unixodbc-dev \
        lintian
fi

if ! command -v cargo &> /dev/null; then
    log_error "Rust/Cargo not found. Install from https://rustup.rs"
    exit 1
fi

log_info "Updating changelog with current version..."
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
sed -i "1s/.*/sqlpage ($VERSION-1) unstable; urgency=medium/" debian/changelog

log_info "Building Debian package..."
dpkg-buildpackage -us -uc -b -d

log_info "Collecting build artifacts..."
mkdir -p build-output
mv ../*.deb build-output/ 2>/dev/null || true
mv ../*.changes build-output/ 2>/dev/null || true
mv ../*.buildinfo build-output/ 2>/dev/null || true

log_info "Running lintian checks..."
lintian --no-tag-display-limit build-output/*.deb || log_warn "Some lintian checks failed"

log_info "Package contents:"
dpkg-deb --contents build-output/*.deb
dpkg-deb --info build-output/*.deb

log_info "Done! Package ready at:"
ls -lh build-output/*.deb
