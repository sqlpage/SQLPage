# SQLPage Build Scripts

This directory contains scripts for building and testing SQLPage packages.

## Available Scripts

### `ci-build-deb.sh`
Builds a Debian/Ubuntu `.deb` package for CI environments.

**Requirements:**
- Debian or Ubuntu system (or container)
- `debhelper`, `dpkg-dev`, `cargo`, and build dependencies

**Usage:**
```bash
./scripts/ci-build-deb.sh
```

**Output:** `build-output/sqlpage_*.deb`

### `ci-build-rpm.sh`
Builds an RPM package for Fedora, RHEL, Rocky Linux, etc. for CI environments.

**Requirements:**
- RPM-based system (or container)
- `rpm-build`, `rpmdevtools`, `cargo`, and build dependencies

**Usage:**
```bash
./scripts/ci-build-rpm.sh
```

**Output:** `~/rpmbuild/RPMS/x86_64/sqlpage*.rpm` and `~/rpmbuild/SRPMS/sqlpage*.rpm`

### `ci-test-package.sh`
Tests package installation on a single distribution.

**Requirements:**
- Package file available
- Appropriate package manager (apt/dnf/yum)

**Usage:**
```bash
./scripts/ci-test-package.sh [package-file]
```

Tests package installation, systemd service, and basic functionality.

## Quick Start

### Building Packages in Docker

**Debian package:**
```bash
docker run -it -v $(pwd):/workspace -w /workspace debian:bookworm bash -c "
  apt-get update && \
  apt-get install -y debhelper cargo rustc unixodbc-dev freetds-dev libssl-dev pkg-config dpkg-dev && \
  ./scripts/ci-build-deb.sh
"
```

**RPM package:**
```bash
docker run -it -v $(pwd):/workspace -w /workspace fedora:latest bash -c "
  dnf install -y rpm-build rpmdevtools rust cargo openssl-devel unixODBC-devel freetds-devel systemd-rpm-macros git && \
  ./scripts/ci-build-rpm.sh
"
```

## CI/CD Integration

These scripts are integrated into GitHub Actions workflows:
- `.github/workflows/packages.yml` - Package building and testing
- `.github/workflows/release.yml` - Release automation

Packages are automatically:
1. Built on every commit to main
2. Tested on multiple distributions
3. Published to GitHub Releases on version tags

## See Also

- [PACKAGING.md](../PACKAGING.md) - Complete packaging documentation
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [.github/workflows/packages.yml](../.github/workflows/packages.yml) - CI configuration

# Docker Build Scripts

This directory contains scripts used by the Dockerfile to build SQLPage with cross-compilation support.

## Scripts

- **`setup-cross-compilation.sh`**: Sets up the cross-compilation environment based on target and build architectures. Handles system dependencies, cross-compiler installation, and libgcc extraction for runtime.
- **`build-dependencies.sh`**: Builds only the project dependencies for Docker layer caching
- **`build-project.sh`**: Builds the final SQLPage binary

## Usage

These scripts are automatically copied and executed by the Dockerfile during the build process. They handle:

- Cross-compilation setup for different architectures (amd64, arm64, arm)
- System dependencies installation
- Cargo build configuration with appropriate linkers
- Library extraction for runtime

The scripts use temporary files in `/tmp/` to pass configuration between stages and export environment variables for use in subsequent build steps.
