#!/bin/bash
set -eux

TARGETARCH="${1:-amd64}"
DUCKDB_VERSION="${2:-v1.4.3.0}"

# Determine the correct DuckDB ODBC package for the architecture
case "$TARGETARCH" in
    amd64) odbc_zip="duckdb_odbc-linux-amd64.zip" ;;
    arm64) odbc_zip="duckdb_odbc-linux-arm64.zip" ;;
    *) echo "Unsupported TARGETARCH: $TARGETARCH" >&2; exit 1 ;;
esac

# Download and install DuckDB ODBC driver
curl -fsSL -o /tmp/duckdb_odbc.zip "https://github.com/duckdb/duckdb-odbc/releases/download/${DUCKDB_VERSION}/${odbc_zip}"
mkdir -p /opt/duckdb_odbc
unzip /tmp/duckdb_odbc.zip -d /opt/duckdb_odbc
rm /tmp/duckdb_odbc.zip

# Configure ODBC driver in odbcinst.ini
cat >> /etc/odbcinst.ini << EOF

[DuckDB]
Description=DuckDB ODBC Driver
Driver=/opt/duckdb_odbc/libduckdb_odbc.so
Setup=/opt/duckdb_odbc/libduckdb_odbc.so
UsageCount=1
EOF

# Configure default DuckDB data source in odbc.ini
cat >> /etc/odbc.ini << EOF

[DuckDB]
Driver=DuckDB
Database=/var/lib/sqlpage/duckdb.db
EOF
