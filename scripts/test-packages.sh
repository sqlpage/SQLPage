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

# Test package using unified test script
test_package() {
    local distro=$1
    local version=$2
    local package_type=$3
    log_info "Testing $package_type package on $distro:$version"

    if [ "$package_type" = "deb" ]; then
        docker run --rm -v "$PROJECT_ROOT/build-output":/packages "$distro:$version" bash -c "
            apt-get update -qq
            PACKAGE_FILE=\$(ls /packages/sqlpage*.deb | grep -v dbgsym | head -1)
            echo \"Installing: \$PACKAGE_FILE\"
            apt-get install -y \"\$PACKAGE_FILE\"
        " && log_success "$package_type test passed on $distro:$version" || log_error "$package_type test failed on $distro:$version"
    elif [ "$package_type" = "rpm" ]; then
        docker run --rm -v "$HOME/rpmbuild":/rpmbuild -v "$PROJECT_ROOT/scripts":/scripts "$distro:$version" bash -c "
            cp /scripts/ci-test-package.sh /tmp/
            chmod +x /tmp/ci-test-package.sh
            cd /tmp
            ./ci-test-package.sh /rpmbuild/RPMS/x86_64/sqlpage*.rpm
        " && log_success "$package_type test passed on $distro:$version" || log_error "$package_type test failed on $distro:$version"
    fi
}

# Main test suite
main() {
    log_info "Starting package installation tests"
    
    # Test DEB packages
    if ls "$PROJECT_ROOT/build-output/sqlpage"*.deb 1> /dev/null 2>&1; then
        log_info "Found DEB package, testing on multiple distributions..."
        test_package "debian" "bookworm" "deb"
        test_package "debian" "bullseye" "deb"
        test_package "ubuntu" "24.04" "deb"
        test_package "ubuntu" "22.04" "deb"
    else
        log_warn "No DEB package found, skipping DEB tests"
    fi

    # Test RPM packages
    if [ -d "$HOME/rpmbuild/RPMS" ] && find "$HOME/rpmbuild/RPMS" -name "sqlpage*.rpm" | grep -q .; then
        log_info "Found RPM package, testing on multiple distributions..."
        test_package "fedora" "latest" "rpm"
        test_package "fedora" "39" "rpm"
        test_package "rockylinux" "9" "rpm"
        test_package "rockylinux" "8" "rpm"
    else
        log_warn "No RPM package found, skipping RPM tests"
    fi
    
    log_info "Package testing complete!"
}

main "$@"
