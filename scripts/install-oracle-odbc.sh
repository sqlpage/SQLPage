#!/usr/bin/env bash
set -euo pipefail

: "${GITHUB_ENV:?Set GITHUB_ENV to the file that should receive the driver environment}"

cd "$(dirname "${BASH_SOURCE[0]}")/.."

# Oracle Instant Client 23ai (23.26.x). 21c is an older innovation release;
# 23ai matches the database server used in CI (gvenzl/oracle-free:23.x)
# and may have fixed or changed the client cleanup path involved in the
# finiSqora exit hang. Download the official ZIPs instead of converting
# RPMs to Debian packages. CI caches the archives; verify them even on
# cache hits.
oracle_version="23.26.0.0.0"
oracle_build="2326000"
oracle_dir="instantclient_23_26"
oracle_driver="Oracle 23 ODBC driver"
oracle_so="libsqora.so.23.1"
archive_dir="${RUNNER_TEMP:-/tmp}/sqlpage-oracle-instantclient-archives"
install_dir="${RUNNER_TEMP:-/tmp}/sqlpage-oracle-instantclient"
mkdir -p "$archive_dir" "$install_dir"

for package in basic odbc; do
  archive="instantclient-${package}-linux.x64-${oracle_version}.zip"
  if [[ ! -f "$archive_dir/$archive" ]]; then
    curl --fail --location --remove-on-error --retry 3 --connect-timeout 15 \
      --max-time 300 --output "$archive_dir/$archive" \
      "https://download.oracle.com/otn_software/linux/instantclient/${oracle_build}/$archive"
  fi
done

(
  cd "$archive_dir"
  sha256sum --check <<'CHECKSUMS'
d6c79cbcf0ff209363e779855c690d4fc730aed847e9198a2c439bcf34760af5  instantclient-basic-linux.x64-23.26.0.0.0.zip
e4e715d2dbf7f1c6907adceb8a62bea33d3ae2ae4466118df346b6c213af4fb4  instantclient-odbc-linux.x64-23.26.0.0.0.zip
CHECKSUMS
)

for package in basic odbc; do
  unzip -oq "$archive_dir/instantclient-${package}-linux.x64-${oracle_version}.zip" -d "$install_dir"
done
client_dir="$install_dir/$oracle_dir"

# Ubuntu 24.04's libaio package uses a different SONAME from Oracle's client.
libaio_path="$(ldconfig -p | awk '$1 ~ /^libaio\.so\.1(t64)?$/ { path = $NF } END { print path }')"
test -n "$libaio_path"
ln -sf "$libaio_path" "$client_dir/libaio.so.1"

cat > "$install_dir/odbcinst.ini" <<EOF
[$oracle_driver]
Description=Oracle ODBC driver for Oracle 23
Driver=$client_dir/$oracle_so
EOF

{
  echo "LD_LIBRARY_PATH=$client_dir:${LD_LIBRARY_PATH:-}"
  echo "ODBCSYSINI=$install_dir"
} >> "$GITHUB_ENV"
