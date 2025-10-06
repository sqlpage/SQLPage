#!/bin/bash
set -euo pipefail

# Detect package manager and set variables accordingly
if command -v apt-get &> /dev/null; then
    PACKAGE_MANAGER="apt"
    INSTALL_CMD="apt-get install -y"
    UPDATE_CMD="apt-get update -qq"
    QUERY_CMD="dpkg -l | grep sqlpage"
    SERVICE_PATH="/lib/systemd/system/sqlpage.service"
elif command -v dnf &> /dev/null; then
    PACKAGE_MANAGER="dnf"
    INSTALL_CMD="dnf install -y"
    UPDATE_CMD=""
    QUERY_CMD="rpm -qi sqlpage"
    SERVICE_PATH="/usr/lib/systemd/system/sqlpage.service"
elif command -v yum &> /dev/null; then
    PACKAGE_MANAGER="yum"
    INSTALL_CMD="yum install -y"
    UPDATE_CMD=""
    QUERY_CMD="rpm -qi sqlpage"
    SERVICE_PATH="/usr/lib/systemd/system/sqlpage.service"
else
    echo "Error: No supported package manager found (apt, dnf, yum)"
    exit 1
fi

PACKAGE_FILE=${1:-sqlpage*.deb}
if [[ $PACKAGE_FILE == *.rpm ]]; then
    PACKAGE_FILE=${PACKAGE_FILE}
elif [[ $PACKAGE_FILE == *.deb ]]; then
    PACKAGE_FILE=${PACKAGE_FILE}
else
    # Auto-detect package type based on available files, prefer main package over debug symbols
    if ls sqlpage*.rpm 1> /dev/null 2>&1; then
        PACKAGE_FILE="sqlpage*.rpm"
    elif ls sqlpage*.deb 1> /dev/null 2>&1; then
        # Find the main package (not debug symbols)
        MAIN_PACKAGE=$(ls sqlpage*.deb | grep -v dbgsym | head -1)
        PACKAGE_FILE="$MAIN_PACKAGE"
    fi
fi

echo "=== Installing package using $PACKAGE_MANAGER ==="
if [[ -n "$UPDATE_CMD" ]]; then
    $UPDATE_CMD
fi
# Install the package, avoiding debug symbols if possible
if [[ $PACKAGE_MANAGER == "apt" ]]; then
    apt-get install -y "$PACKAGE_FILE" --no-install-recommends || apt-get install -y "$PACKAGE_FILE"
else
    $INSTALL_CMD "$PACKAGE_FILE"
fi

echo "=== Verifying installation ==="
sqlpage --version
which sqlpage
$QUERY_CMD

echo "=== Checking files ==="
test -f /usr/bin/sqlpage
test -d /etc/sqlpage
test -f /etc/sqlpage/sqlpage.json
test -d /etc/sqlpage/templates
test -f "$SERVICE_PATH"
test -f /usr/share/man/man1/sqlpage.1.gz

echo "=== Verifying systemd service ==="
systemctl cat sqlpage.service || cat "$SERVICE_PATH"

echo "=== Verifying user ==="
id sqlpage
getent passwd sqlpage

echo "=== Testing functionality with systemd ==="
echo "SELECT 'json' as component; SELECT 1 as it_works;" > /var/www/sqlpage/index.sql

# Enable and start the service
systemctl enable sqlpage
systemctl start sqlpage

# Wait for service to start
sleep 3

# Test if the web interface works
curl -sf http://localhost:8080/ | grep -q it_works

# Clean up
systemctl stop sqlpage
systemctl disable sqlpage

echo "✓ All package tests passed!"
