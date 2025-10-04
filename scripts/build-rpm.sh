#!/bin/bash
set -euo pipefail

# Build RPM package for SQLPage
# This script builds an .rpm package following RPM best practices

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

# Check if we're on an RPM-based system
if ! command -v rpmbuild &> /dev/null; then
    log_error "rpmbuild not found. Install it with: yum install rpm-build or dnf install rpm-build"
    exit 1
fi

# Check for required tools
MISSING_DEPS=()
if ! command -v cargo &> /dev/null; then
    MISSING_DEPS+=("cargo")
fi
if ! command -v rpmdev-setuptree &> /dev/null; then
    MISSING_DEPS+=("rpmdevtools")
fi

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    log_error "Missing dependencies: ${MISSING_DEPS[*]}"
    log_info "Install them with: yum install ${MISSING_DEPS[*]} or dnf install ${MISSING_DEPS[*]}"
    exit 1
fi

cd "$PROJECT_ROOT"

# Setup RPM build tree
log_info "Setting up RPM build tree..."
rpmdev-setuptree

# Get version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
log_info "Building RPM for version $VERSION"

# Copy spec file
log_info "Copying spec file..."
cp rpm/sqlpage.spec ~/rpmbuild/SPECS/

# Create source tarball
log_info "Creating source tarball..."
TARBALL="sqlpage-${VERSION}.tar.gz"
git archive --format=tar.gz --prefix="SQLPage-${VERSION}/" -o ~/rpmbuild/SOURCES/"$TARBALL" HEAD

# Build the RPM
log_info "Building RPM package..."
rpmbuild -ba ~/rpmbuild/SPECS/sqlpage.spec

log_info "Package built successfully!"
log_info "Output files:"
find ~/rpmbuild/RPMS -name "sqlpage*.rpm" -exec ls -lh {} \;
find ~/rpmbuild/SRPMS -name "sqlpage*.rpm" -exec ls -lh {} \;

# Run rpmlint if available
if command -v rpmlint &> /dev/null; then
    log_info "Running rpmlint checks..."
    rpmlint ~/rpmbuild/RPMS/*/sqlpage*.rpm || log_warn "Some rpmlint checks failed (this may be acceptable)"
fi

log_info "Done! Packages are ready at:"
find ~/rpmbuild/RPMS -name "sqlpage*.rpm" -o -path ~/rpmbuild/SRPMS -name "sqlpage*.rpm"
