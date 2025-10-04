#!/bin/bash
set -euo pipefail

# Test package installation on various distributions using Docker
# This script validates that the packages install and run correctly

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

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Test DEB package on Debian/Ubuntu
test_deb() {
    local distro=$1
    local version=$2
    log_info "Testing DEB package on $distro:$version"
    
    docker run --rm -v "$PROJECT_ROOT":/workspace "$distro:$version" bash -c "
        set -e
        apt-get update
        apt-get install -y /workspace/../sqlpage*.deb
        sqlpage --version
        systemctl --version || true
        dpkg -l | grep sqlpage
        dpkg -L sqlpage
    " && log_success "DEB test passed on $distro:$version" || log_error "DEB test failed on $distro:$version"
}

# Test RPM package on Fedora/RHEL
test_rpm() {
    local distro=$1
    local version=$2
    log_info "Testing RPM package on $distro:$version"
    
    docker run --rm -v "$HOME/rpmbuild":/rpmbuild "$distro:$version" bash -c "
        set -e
        yum install -y /rpmbuild/RPMS/x86_64/sqlpage*.rpm || dnf install -y /rpmbuild/RPMS/x86_64/sqlpage*.rpm
        sqlpage --version
        systemctl --version || true
        rpm -qi sqlpage
        rpm -ql sqlpage
    " && log_success "RPM test passed on $distro:$version" || log_error "RPM test failed on $distro:$version"
}

# Main test suite
main() {
    log_info "Starting package installation tests"
    
    # Test DEB packages
    if [ -f "$PROJECT_ROOT/../sqlpage"*.deb ]; then
        log_info "Found DEB package, testing on multiple distributions..."
        test_deb "debian" "bookworm"
        test_deb "debian" "bullseye"
        test_deb "ubuntu" "24.04"
        test_deb "ubuntu" "22.04"
    else
        log_warn "No DEB package found, skipping DEB tests"
    fi
    
    # Test RPM packages
    if [ -d "$HOME/rpmbuild/RPMS" ] && find "$HOME/rpmbuild/RPMS" -name "sqlpage*.rpm" | grep -q .; then
        log_info "Found RPM package, testing on multiple distributions..."
        test_rpm "fedora" "latest"
        test_rpm "fedora" "39"
        test_rpm "rockylinux" "9"
        test_rpm "rockylinux" "8"
    else
        log_warn "No RPM package found, skipping RPM tests"
    fi
    
    log_info "Package testing complete!"
}

main "$@"
