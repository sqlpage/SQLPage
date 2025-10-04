# SQLPage Build Scripts

This directory contains scripts for building and testing SQLPage packages.

## Available Scripts

### `build-deb.sh`
Builds a Debian/Ubuntu `.deb` package.

**Requirements:**
- Debian or Ubuntu system (or container)
- `debhelper`, `dpkg-dev`, `cargo`, and build dependencies

**Usage:**
```bash
./scripts/build-deb.sh
```

**Output:** `../sqlpage_*.deb`

### `build-rpm.sh`
Builds an RPM package for Fedora, RHEL, Rocky Linux, etc.

**Requirements:**
- RPM-based system (or container)
- `rpm-build`, `rpmdevtools`, `cargo`, and build dependencies

**Usage:**
```bash
./scripts/build-rpm.sh
```

**Output:** `~/rpmbuild/RPMS/x86_64/sqlpage*.rpm` and `~/rpmbuild/SRPMS/sqlpage*.rpm`

### `test-packages.sh`
Tests package installation across multiple distributions using Docker.

**Requirements:**
- Docker installed and running
- Built packages available

**Usage:**
```bash
./scripts/test-packages.sh
```

Tests packages on:
- Debian: bookworm, bullseye
- Ubuntu: 24.04, 22.04
- Fedora: latest, 39
- Rocky Linux: 9, 8

## Quick Start

### Building Both Package Types in Docker

**Debian package:**
```bash
docker run -it -v $(pwd):/workspace -w /workspace debian:bookworm bash -c "
  apt-get update && \
  apt-get install -y debhelper cargo rustc unixodbc-dev freetds-dev libssl-dev pkg-config dpkg-dev && \
  ./scripts/build-deb.sh
"
```

**RPM package:**
```bash
docker run -it -v $(pwd):/workspace -w /workspace fedora:latest bash -c "
  dnf install -y rpm-build rpmdevtools rust cargo openssl-devel unixODBC-devel freetds-devel systemd-rpm-macros git && \
  ./scripts/build-rpm.sh
"
```

## CI/CD Integration

These scripts are integrated into GitHub Actions workflows:
- `.github/workflows/packages.yml` - Main package building and testing
- `.github/workflows/release.yml` - Release automation

Packages are automatically:
1. Built on every commit to main
2. Tested on multiple distributions
3. Published to GitHub Releases on version tags

## See Also

- [PACKAGING.md](../PACKAGING.md) - Complete packaging documentation
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [.github/workflows/packages.yml](../.github/workflows/packages.yml) - CI configuration
