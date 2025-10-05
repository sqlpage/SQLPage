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
if command -v dnf &> /dev/null; then
    sudo dnf install -y rpm-build rpmdevtools openssl-devel systemd unixODBC-devel
elif command -v yum &> /dev/null; then
    sudo yum install -y rpm-build rpmdevtools openssl-devel systemd unixODBC-devel
fi

if ! command -v cargo &> /dev/null; then
    log_error "Rust/Cargo not found. Install from https://rustup.rs"
    exit 1
fi

log_info "Setting up RPM build tree..."
rpmdev-setuptree

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
RPM_VERSION=$(echo "$VERSION" | sed 's/-/~/')
log_info "Building RPM for version $VERSION (RPM: $RPM_VERSION)"

log_info "Updating spec file version..."
sed -i "s/^Version:.*/Version:        ${RPM_VERSION}/" rpm/sqlpage.spec
cp rpm/sqlpage.spec ~/rpmbuild/SPECS/

log_info "Creating source tarball..."
git archive --format=tar.gz --prefix="SQLPage-${RPM_VERSION}/" \
    -o ~/rpmbuild/SOURCES/v${RPM_VERSION}.tar.gz HEAD

log_info "Building RPM package..."
rpmbuild -ba --nodeps ~/rpmbuild/SPECS/sqlpage.spec

log_info "Running rpmlint checks..."
rpmlint ~/rpmbuild/RPMS/*/sqlpage*.rpm || log_warn "Some rpmlint checks failed"
rpmlint ~/rpmbuild/SRPMS/sqlpage*.rpm || true

log_info "Package contents:"
rpm -qilp ~/rpmbuild/RPMS/*/sqlpage*.rpm

log_info "Done! Packages ready at:"
find ~/rpmbuild/RPMS ~/rpmbuild/SRPMS -name "sqlpage*.rpm" -exec ls -lh {} \;
