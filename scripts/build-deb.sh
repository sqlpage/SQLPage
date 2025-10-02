#!/bin/bash
set -euo pipefail

# Build Debian package for SQLPage
# This script builds a .deb package following Debian best practices

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're on a Debian-based system
if ! command -v dpkg-buildpackage &> /dev/null; then
    log_error "dpkg-buildpackage not found. Install it with: apt-get install dpkg-dev"
    exit 1
fi

# Check for required tools
MISSING_DEPS=()
if ! command -v debhelper &> /dev/null; then
    MISSING_DEPS+=("debhelper")
fi
if ! command -v cargo &> /dev/null; then
    MISSING_DEPS+=("cargo")
fi

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    log_error "Missing dependencies: ${MISSING_DEPS[*]}"
    log_info "Install them with: apt-get install ${MISSING_DEPS[*]}"
    exit 1
fi

cd "$PROJECT_ROOT"

log_info "Cleaning previous builds..."
rm -rf debian/.debhelper debian/sqlpage debian/cargo_home
cargo clean || true

log_info "Building Debian package..."
dpkg-buildpackage -us -uc -b

log_info "Package built successfully!"
log_info "Output files:"
ls -lh ../*.deb || true

# Run lintian if available
if command -v lintian &> /dev/null; then
    log_info "Running lintian checks..."
    lintian ../*.deb || log_warn "Some lintian checks failed (this may be acceptable)"
fi

log_info "Done! Package is ready at:"
readlink -f ../*.deb || true
