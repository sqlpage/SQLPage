# SQLPage Packaging Guide

This document describes the packaging system for SQLPage, including how to build, test, and distribute DEB and RPM packages.

## Overview

SQLPage provides native packages for:
- **Debian/Ubuntu** - `.deb` packages
- **Fedora/RHEL/Rocky/Alma** - `.rpm` packages

All packages follow distribution best practices and are automatically built and tested in CI on every release.

## Package Features

### Common Features

- ✅ Systemd service integration
- ✅ Dedicated `sqlpage` system user
- ✅ Automatic ODBC dependency installation
- ✅ Configuration directory at `/etc/sqlpage`
- ✅ Web root at `/var/www/sqlpage`
- ✅ Secure file permissions
- ✅ Clean uninstall with optional data preservation

### Debian Package (`sqlpage.deb`)

**Installation:**
```bash
sudo apt install ./sqlpage_*.deb
```

**Files installed:**
- `/usr/bin/sqlpage` - Main binary
- `/etc/sqlpage/` - Configuration and templates
- `/var/www/sqlpage/` - Web root directory
- `/lib/systemd/system/sqlpage.service` - Systemd service
- `/var/log/sqlpage/` - Log directory

**Dependencies:**
- `unixodbc` (required)
- `sqlite3` or `postgresql-client` or `mariadb-client` (recommended)

### RPM Package (`sqlpage.rpm`)

**Installation:**
```bash
sudo yum install sqlpage-*.rpm    # RHEL/CentOS/Rocky/Alma
sudo dnf install sqlpage-*.rpm    # Fedora
```

**Files installed:**
- `/usr/bin/sqlpage` - Main binary
- `/etc/sqlpage/` - Configuration and templates
- `/var/www/sqlpage/` - Web root directory
- `/usr/lib/systemd/system/sqlpage.service` - Systemd service
- `/var/log/sqlpage/` - Log directory

**Dependencies:**
- `unixODBC` (required)
- `sqlite`, `postgresql`, or `mariadb` (recommended)

## Building Packages

### Prerequisites

#### For Debian packages:
```bash
sudo apt-get install -y \
    debhelper \
    dh-make \
    devscripts \
    lintian \
    dpkg-dev \
    build-essential \
    cargo \
    rustc \
    unixodbc-dev \
    freetds-dev \
    libssl-dev \
    pkg-config
```

#### For RPM packages:
```bash
# Fedora/RHEL 8+
sudo dnf install -y \
    rpm-build \
    rpmdevtools \
    rpmlint \
    rust \
    cargo \
    openssl-devel \
    systemd-rpm-macros \
    unixODBC-devel \
    freetds-devel

# RHEL/CentOS 7
sudo yum install -y \
    rpm-build \
    rpmdevtools \
    rpmlint \
    rust \
    cargo \
    openssl-devel \
    systemd \
    unixODBC-devel \
    freetds-devel
```

### Build Scripts

#### Debian Package

```bash
./scripts/build-deb.sh
```

This script:
1. Cleans previous builds
2. Runs `dpkg-buildpackage` to create the package
3. Runs `lintian` for quality checks
4. Outputs package to `../sqlpage_*.deb`

#### RPM Package

```bash
./scripts/build-rpm.sh
```

This script:
1. Sets up RPM build tree
2. Creates source tarball
3. Builds binary and source RPMs
4. Runs `rpmlint` for quality checks
5. Outputs packages to `~/rpmbuild/RPMS/` and `~/rpmbuild/SRPMS/`

### Manual Building

#### Debian (manual)

```bash
# Update changelog if needed
dch -v $(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')-1 "New release"

# Build package
dpkg-buildpackage -us -uc -b

# Check package
lintian ../sqlpage_*.deb
```

#### RPM (manual)

```bash
# Set up build tree
rpmdev-setuptree

# Copy spec file
cp rpm/sqlpage.spec ~/rpmbuild/SPECS/

# Create source tarball
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
git archive --format=tar.gz --prefix="SQLPage-${VERSION}/" \
    -o ~/rpmbuild/SOURCES/sqlpage-${VERSION}.tar.gz HEAD

# Build RPM
rpmbuild -ba ~/rpmbuild/SPECS/sqlpage.spec

# Check package
rpmlint ~/rpmbuild/RPMS/*/sqlpage*.rpm
```

## Testing Packages

### Automated Testing

The `scripts/test-packages.sh` script tests package installation across multiple distributions:

```bash
./scripts/test-packages.sh
```

This uses Docker to test on:
- Debian: bookworm, bullseye
- Ubuntu: 24.04, 22.04, 20.04
- Fedora: latest, 39, 40
- Rocky Linux: 9, 8
- AlmaLinux: 9, 8

### Manual Testing

#### Test Debian package:
```bash
docker run -it -v $(pwd)/..:/packages debian:bookworm bash
apt update
apt install -y /packages/sqlpage_*.deb
sqlpage --version
systemctl cat sqlpage
```

#### Test RPM package:
```bash
docker run -it -v ~/rpmbuild:/rpmbuild fedora:latest bash
dnf install -y /rpmbuild/RPMS/x86_64/sqlpage*.rpm
sqlpage --version
systemctl cat sqlpage
```

## CI/CD Integration

### Automatic Building

Packages are automatically built on:
- Every push to `main` branch (for testing)
- Every tag matching `v*` (for release)
- Pull requests that modify packaging files

See `.github/workflows/packages.yml` for details.

### Testing Matrix

Each package is tested on multiple distributions:

| Distribution | Versions | Package Type |
|-------------|----------|--------------|
| Debian | bookworm, bullseye | DEB |
| Ubuntu | 24.04, 22.04, 20.04 | DEB |
| Fedora | latest, 39, 40 | RPM |
| Rocky Linux | 9, 8 | RPM |
| AlmaLinux | 9, 8 | RPM |

### Release Process

When a new tag is pushed:
1. Packages are built for all platforms
2. Packages are tested on all distributions
3. On success, packages are uploaded to GitHub Release
4. Checksums (SHA256SUMS) are generated

## Usage

### Installing SQLPage

#### Debian/Ubuntu:
```bash
# Download from GitHub releases
wget https://github.com/sqlpage/SQLPage/releases/latest/download/sqlpage_*.deb

# Install
sudo apt install ./sqlpage_*.deb
```

#### Fedora/RHEL:
```bash
# Download from GitHub releases
wget https://github.com/sqlpage/SQLPage/releases/latest/download/sqlpage-*.rpm

# Install
sudo dnf install ./sqlpage-*.rpm  # Fedora
sudo yum install ./sqlpage-*.rpm  # RHEL/Rocky/Alma
```

### Starting the Service

```bash
# Enable and start
sudo systemctl enable sqlpage
sudo systemctl start sqlpage

# Check status
sudo systemctl status sqlpage

# View logs
sudo journalctl -u sqlpage -f
```

### Configuration

1. Edit `/etc/sqlpage/sqlpage.json` for configuration
2. Place SQL files in `/var/www/sqlpage/`
3. Restart service: `sudo systemctl restart sqlpage`

### Uninstalling

#### Debian/Ubuntu:
```bash
# Remove package but keep configuration
sudo apt remove sqlpage

# Remove everything including configuration
sudo apt purge sqlpage
```

#### Fedora/RHEL:
```bash
# Remove package
sudo yum remove sqlpage  # or dnf remove sqlpage

# Configuration files are marked as %config(noreplace)
# and won't be removed automatically
```

## Package Maintenance

### Updating Version

1. Update `Cargo.toml` version
2. Update `debian/changelog`:
   ```bash
   dch -v NEW_VERSION-1 "Release notes"
   ```
3. Update `rpm/sqlpage.spec` version field
4. Commit and tag: `git tag vNEW_VERSION`

### Adding Dependencies

#### Debian:
- Edit `debian/control` - add to `Build-Depends` or `Depends`

#### RPM:
- Edit `rpm/sqlpage.spec` - add to `BuildRequires` or `Requires`

### Modifying File Installation

#### Debian:
- Edit `debian/rules` - modify `override_dh_auto_install` target
- Edit `debian/install` - add files to be installed

#### RPM:
- Edit `rpm/sqlpage.spec` - modify `%install` and `%files` sections

## Best Practices

### Security
- Packages run as dedicated `sqlpage` user (not root)
- Systemd security hardening enabled
- Proper file permissions enforced
- Log directory properly secured

### Compatibility
- Uses manylinux for maximum compatibility
- Static linking where possible
- ODBC support for multiple databases
- Systemd integration for modern distros

### Quality
- Lintian checks for Debian packages
- Rpmlint checks for RPM packages
- Installation tested on multiple distributions
- Service file validates correctly
- User creation handled properly

## Troubleshooting

### Build Failures

**Cargo/Rust not found:**
```bash
# Debian/Ubuntu
sudo apt install cargo rustc

# Fedora/RHEL
sudo dnf install rust cargo
```

**ODBC headers not found:**
```bash
# Debian/Ubuntu
sudo apt install unixodbc-dev

# Fedora/RHEL
sudo dnf install unixODBC-devel
```

### Installation Issues

**Dependency conflicts:**
- Check distribution compatibility
- Verify ODBC libraries are available
- Try `--force-depends` (not recommended for production)

**Service won't start:**
```bash
# Check logs
sudo journalctl -u sqlpage -n 50

# Verify configuration
sudo -u sqlpage /usr/bin/sqlpage --version

# Check permissions
ls -la /var/www/sqlpage /etc/sqlpage
```

## Contributing

When contributing packaging changes:

1. Test on multiple distributions
2. Run lintian/rpmlint checks
3. Update this documentation
4. Test upgrade path from previous version
5. Ensure CI passes

## Support

For packaging issues:
- GitHub Issues: https://github.com/sqlpage/SQLPage/issues
- Documentation: https://sql-page.com
- Community: See CONTRIBUTING.md

## License

The packaging scripts and configurations are provided under the same MIT license as SQLPage itself.
