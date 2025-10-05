#!/bin/bash
set -euo pipefail

PACKAGE_FILE=${1:-sqlpage*.deb}

echo "=== Installing package ==="
apt-get update -qq
apt-get install -y "$PACKAGE_FILE"

echo "=== Verifying installation ==="
sqlpage --version
which sqlpage
dpkg -l | grep sqlpage

echo "=== Checking files ==="
test -f /usr/bin/sqlpage
test -d /etc/sqlpage
test -f /etc/sqlpage/sqlpage.json
test -d /etc/sqlpage/templates
test -f /lib/systemd/system/sqlpage.service
test -f /usr/share/man/man1/sqlpage.1.gz

echo "=== Verifying systemd service ==="
systemctl cat sqlpage.service || cat /lib/systemd/system/sqlpage.service

echo "=== Verifying user ==="
id sqlpage
getent passwd sqlpage

echo "=== Testing functionality ==="
echo "SELECT 'json' as component; SELECT 1 as it_works;" > /var/www/sqlpage/index.sql
cd /var/www/sqlpage
timeout 5 sqlpage &
sleep 2
curl -sf http://localhost:8080/ | grep -q it_works
pkill sqlpage || true

echo "✓ All DEB tests passed!"
