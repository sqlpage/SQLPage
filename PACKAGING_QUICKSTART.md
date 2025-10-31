# SQLPage Packaging - Quick Start

## 🚀 Quick Commands

### Build Packages Locally

```bash
# Debian/Ubuntu
./scripts/build-deb.sh

# Fedora/RHEL/Rocky/Alma
./scripts/build-rpm.sh

# Validate configuration
./scripts/validate-packaging.sh

# Test on multiple distributions
./scripts/test-packages.sh
```

### Install Packages

```bash
# Debian/Ubuntu
sudo apt install ./sqlpage_*.deb

# Fedora/RHEL/Rocky/Alma  
sudo dnf install ./sqlpage-*.rpm

# Start service
sudo systemctl enable --now sqlpage
sudo systemctl status sqlpage
```

## 📦 What Gets Installed

| Path | Purpose |
|------|---------|
| `/usr/bin/sqlpage` | Main executable |
| `/etc/sqlpage/` | Configuration & templates |
| `/var/www/sqlpage/` | Web root for SQL files |
| `/var/log/sqlpage/` | Log directory |
| `/lib/systemd/system/sqlpage.service` | Systemd unit |

## 🔧 Configuration

Edit `/etc/sqlpage/sqlpage.json`:

```json
{
  "listen_on": "0.0.0.0:8080",
  "database_url": "sqlite:///var/www/sqlpage/sqlpage.db"
}
```

Then restart: `sudo systemctl restart sqlpage`

## 📝 Add SQL Files

```bash
sudo -u sqlpage nano /var/www/sqlpage/index.sql
```

## 📊 View Logs

```bash
# Follow logs
sudo journalctl -u sqlpage -f

# Last 100 lines
sudo journalctl -u sqlpage -n 100
```

## 🔄 Update Package

```bash
# Download new version
wget https://github.com/sqlpage/SQLPage/releases/latest/download/sqlpage_*.deb
# or
wget https://github.com/sqlpage/SQLPage/releases/latest/download/sqlpage-*.rpm

# Update (Debian/Ubuntu)
sudo apt install ./sqlpage_*.deb

# Update (Fedora/RHEL)
sudo dnf update ./sqlpage-*.rpm
```

## 🗑️ Uninstall

```bash
# Debian/Ubuntu (keep config)
sudo apt remove sqlpage

# Debian/Ubuntu (remove everything)
sudo apt purge sqlpage

# Fedora/RHEL
sudo dnf remove sqlpage
```

## 🐛 Troubleshooting

### Service won't start
```bash
sudo systemctl status sqlpage
sudo journalctl -u sqlpage -n 50
```

### Permission issues
```bash
sudo chown -R sqlpage:sqlpage /var/www/sqlpage
```

### Port in use
Edit `/etc/sqlpage/sqlpage.json` and change port, then:
```bash
sudo systemctl restart sqlpage
```

## 📚 Full Documentation

- [PACKAGING.md](PACKAGING.md) - Complete guide
- [docs/INSTALLATION_PACKAGES.md](docs/INSTALLATION_PACKAGES.md) - User guide
- [configuration.md](configuration.md) - All configuration options

## 🆘 Get Help

- Documentation: https://sql-page.com
- Issues: https://github.com/sqlpage/SQLPage/issues
