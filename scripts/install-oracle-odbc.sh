#!/usr/bin/env bash
set -euo pipefail

: "${GITHUB_ENV:?Set GITHUB_ENV to the file that should receive the driver environment}"

cd "$(dirname "${BASH_SOURCE[0]}")/.."

# Keep the same client version as the RPM installation, without converting RPMs
# to Debian packages. CI caches the archives; verify them even on cache hits.
archive_dir="${RUNNER_TEMP:-/tmp}/sqlpage-oracle-instantclient-archives"
install_dir="${RUNNER_TEMP:-/tmp}/sqlpage-oracle-instantclient"
mkdir -p "$archive_dir" "$install_dir"

for package in basic odbc; do
  archive="instantclient-${package}-linux.x64-21.21.0.0.0dbru.zip"
  if [[ ! -f "$archive_dir/$archive" ]]; then
    curl --fail --location --remove-on-error --retry 3 --connect-timeout 15 \
      --max-time 180 --output "$archive_dir/$archive" \
      "https://download.oracle.com/otn_software/linux/instantclient/2121000/$archive"
  fi
done

(
  cd "$archive_dir"
  sha256sum --check <<'CHECKSUMS'
9cd0d5d5619ddaac43aa2214bab48e84155ca7e057d937634d1909b298125e8a  instantclient-basic-linux.x64-21.21.0.0.0dbru.zip
37e4326ac14b08d9130d499fe5c1ba58f8ad72e8196f5618c0dc5880a24d6a23  instantclient-odbc-linux.x64-21.21.0.0.0dbru.zip
CHECKSUMS
)

for package in basic odbc; do
  unzip -oq "$archive_dir/instantclient-${package}-linux.x64-21.21.0.0.0dbru.zip" -d "$install_dir"
done
client_dir="$install_dir/instantclient_21_21"

# Ubuntu 24.04's libaio package uses a different SONAME from Oracle's client.
libaio_path="$(ldconfig -p | awk '$1 ~ /^libaio\.so\.1(t64)?$/ { path = $NF } END { print path }')"
test -n "$libaio_path"
ln -sf "$libaio_path" "$client_dir/libaio.so.1"

cat > "$install_dir/odbcinst.ini" <<EOF
[Oracle 21 ODBC driver]
Description=Oracle ODBC driver for Oracle 21
Driver=$client_dir/libsqora.so.21.1
EOF

{
  echo "LD_LIBRARY_PATH=$client_dir:${LD_LIBRARY_PATH:-}"
  echo "ODBCSYSINI=$install_dir"
} >> "$GITHUB_ENV"
