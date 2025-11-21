# Debian Packaging for SQLPage

This directory contains the source files for building Debian packages of SQLPage.

## Files

- `control` - Package metadata and dependencies
- `changelog` - Version history for debian packaging
- `copyright` - License and copyright information
- `rules` - Build instructions
- `install` - Files to install and their destinations
- `postinst` - Post-installation script
- `postrm` - Post-removal script
- `sqlpage.service` - systemd service file for package installations

## systemd Service Files

There are **two** systemd service files in this repository:

1. **`/sqlpage.service`** (repository root)
   - For manual/source installations
   - Uses `/usr/local/bin/sqlpage.bin`
   - Includes `RUST_LOG` and `LISTEN_ON` environment variables
   - Includes `AmbientCapabilities=CAP_NET_BIND_SERVICE` for port 80 binding

2. **`/debian/sqlpage.service`** (this directory)
   - For Debian/Ubuntu package installations
   - Uses `/usr/bin/sqlpage` (FHS standard location)
   - Includes `SQLPAGE_CONFIGURATION_DIRECTORY` and `SQLPAGE_WEB_ROOT` variables
   - Does not bind to privileged ports by default

Both files share the same security hardening settings but are customized for their respective installation methods.

## Building

To build the Debian package:

```bash
dpkg-buildpackage -us -uc
```

The built `.deb` file will be placed in the parent directory.

## Testing

After building, you can test the package installation:

```bash
sudo dpkg -i ../sqlpage_*.deb
```

