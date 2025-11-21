# Installing SQLPage from Packages

SQLPage provides native packages for easy installation on Linux systems.

## Debian/Ubuntu (DEB Package)

### Quick Install

```bash
# Download the latest release
wget https://github.com/sqlpage/SQLPage/releases/latest/download/sqlpage_0.38.0-1_amd64.deb

# Install the package
sudo apt install ./sqlpage_0.38.0-1_amd64.deb
```

### Verify Installation

```bash
sqlpage --version
```

### Start the Service

```bash
# Enable and start SQLPage
sudo systemctl enable sqlpage
sudo systemctl start sqlpage

# Check status
sudo systemctl status sqlpage

# View logs
sudo journalctl -u sqlpage -f
```

### Configuration

1. Edit the configuration file:
   ```bash
   sudo nano /etc/sqlpage/sqlpage.json
   ```

2. Add your SQL files to:
   ```bash
   /var/www/sqlpage/
   ```

3. Restart the service:
   ```bash
   sudo systemctl restart sqlpage
   ```

### Supported Distributions

- ✅ Debian 11 (Bullseye)
- ✅ Debian 12 (Bookworm)
- ✅ Ubuntu 20.04 LTS (Focal)
- ✅ Ubuntu 22.04 LTS (Jammy)
- ✅ Ubuntu 24.04 LTS (Noble)

## Fedora/RHEL/Rocky/Alma (RPM Package)

### Quick Install

```bash
# Download the latest release
wget https://github.com/sqlpage/SQLPage/releases/latest/download/sqlpage-0.38.0-1.x86_64.rpm

# Install the package
sudo dnf install ./sqlpage-0.38.0-1.x86_64.rpm    # Fedora/RHEL 8+
# OR
sudo yum install ./sqlpage-0.38.0-1.x86_64.rpm    # RHEL 7/CentOS 7
```

### Verify Installation

```bash
sqlpage --version
```

### Start the Service

```bash
# Enable and start SQLPage
sudo systemctl enable sqlpage
sudo systemctl start sqlpage

# Check status
sudo systemctl status sqlpage

# View logs
sudo journalctl -u sqlpage -f
```

### Configuration

1. Edit the configuration file:
   ```bash
   sudo nano /etc/sqlpage/sqlpage.json
   ```

2. Add your SQL files to:
   ```bash
   /var/www/sqlpage/
   ```

3. Restart the service:
   ```bash
   sudo systemctl restart sqlpage
   ```

### Supported Distributions

- ✅ Fedora 39, 40, 41+
- ✅ RHEL 8, 9
- ✅ Rocky Linux 8, 9
- ✅ AlmaLinux 8, 9
- ✅ CentOS Stream 8, 9

## Package Contents

All packages install the following:

| Item | Location | Description |
|------|----------|-------------|
| Binary | `/usr/bin/sqlpage` | SQLPage executable |
| Configuration | `/etc/sqlpage/` | Configuration files and templates |
| Web Root | `/var/www/sqlpage/` | Default directory for SQL files |
| Service | `/lib/systemd/system/sqlpage.service` | Systemd service unit |
| Logs | `/var/log/sqlpage/` | Log directory |
| User | `sqlpage` | Dedicated system user |

## Firewall Configuration

SQLPage listens on port 8080 by default. Open the firewall:

### Debian/Ubuntu (UFW)
```bash
sudo ufw allow 8080/tcp
```

### Fedora/RHEL (firewalld)
```bash
sudo firewall-cmd --permanent --add-port=8080/tcp
sudo firewall-cmd --reload
```

## Database Setup

### SQLite (Built-in)
No additional setup needed. SQLite is included.

### PostgreSQL
```bash
# Debian/Ubuntu
sudo apt install postgresql-client

# Fedora/RHEL
sudo dnf install postgresql
```

### MySQL/MariaDB
```bash
# Debian/Ubuntu
sudo apt install mariadb-client

# Fedora/RHEL
sudo dnf install mariadb
```

### Other Databases (via ODBC)

For databases like Oracle, Snowflake, ClickHouse, etc., install ODBC drivers:

**Debian/Ubuntu:**
```bash
sudo apt install unixodbc
# Then install your database-specific ODBC driver
```

**Fedora/RHEL:**
```bash
sudo dnf install unixODBC
# Then install your database-specific ODBC driver
```

## Configuration Examples

### Basic Configuration

Edit `/etc/sqlpage/sqlpage.json`:

```json
{
  "listen_on": "0.0.0.0:8080",
  "database_url": "sqlite:///var/www/sqlpage/sqlpage.db"
}
```

### PostgreSQL Configuration

```json
{
  "listen_on": "0.0.0.0:8080",
  "database_url": "postgresql://user:password@localhost/dbname"
}
```

### MySQL Configuration

```json
{
  "listen_on": "0.0.0.0:8080",
  "database_url": "mysql://user:password@localhost/dbname"
}
```

## Updating

### Debian/Ubuntu
```bash
# Download new version
wget https://github.com/sqlpage/SQLPage/releases/latest/download/sqlpage_VERSION.deb

# Update
sudo apt install ./sqlpage_VERSION.deb
```

### Fedora/RHEL
```bash
# Download new version
wget https://github.com/sqlpage/SQLPage/releases/latest/download/sqlpage-VERSION.rpm

# Update
sudo dnf update ./sqlpage-VERSION.rpm
```

## Uninstalling

### Debian/Ubuntu

```bash
# Remove package but keep configuration
sudo apt remove sqlpage

# Remove everything including configuration and data
sudo apt purge sqlpage
```

### Fedora/RHEL

```bash
# Remove package
sudo dnf remove sqlpage

# Configuration files are preserved
# Manually remove them if needed:
sudo rm -rf /etc/sqlpage
```

## Troubleshooting

### Service Won't Start

```bash
# Check service status
sudo systemctl status sqlpage

# View detailed logs
sudo journalctl -u sqlpage -n 100 --no-pager

# Check configuration
sudo -u sqlpage /usr/bin/sqlpage --version
```

### Permission Issues

```bash
# Fix ownership
sudo chown -R sqlpage:sqlpage /var/www/sqlpage
sudo chown -R sqlpage:sqlpage /var/log/sqlpage
```

### Port Already in Use

Edit `/etc/sqlpage/sqlpage.json` and change the port:

```json
{
  "listen_on": "0.0.0.0:8081"
}
```

Then restart:
```bash
sudo systemctl restart sqlpage
```

### Database Connection Issues

Test database connectivity:

```bash
# PostgreSQL
psql -h localhost -U user -d dbname

# MySQL
mysql -h localhost -u user -p dbname

# SQLite
sqlite3 /var/www/sqlpage/sqlpage.db
```

## Advanced Usage

### Custom Configuration Location

Override the configuration directory:

```bash
sudo systemctl edit sqlpage
```

Add:
```ini
[Service]
Environment="SQLPAGE_CONFIGURATION_DIRECTORY=/custom/path"
```

### Running Behind a Reverse Proxy

Example Nginx configuration:

```nginx
server {
    listen 80;
    server_name example.com;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Using HTTPS

SQLPage supports automatic HTTPS with Let's Encrypt. Edit `/etc/sqlpage/sqlpage.json`:

```json
{
  "https_domain": "example.com",
  "https_certificate_email": "admin@example.com"
}
```

## Getting Help

- **Documentation:** https://sql-page.com
- **GitHub Issues:** https://github.com/sqlpage/SQLPage/issues
- **Packaging Guide:** See [PACKAGING.md](../PACKAGING.md) in the repository

## See Also

- [Configuration Guide](../configuration.md)
- [Packaging Documentation](../PACKAGING.md)
- [Main README](../README.md)
