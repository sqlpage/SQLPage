#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cache_dir="target/sqlpage_artefacts"
mkdir -p "$cache_dir"

echo "[1/3] Prefetching Cargo dependencies"
cargo fetch --locked

echo "[2/3] Prefetching npm dependencies"
npm ci --ignore-scripts --no-audit
if [ -f tests/end-to-end/package-lock.json ]; then
    (
        cd tests/end-to-end
        npm ci --ignore-scripts --no-audit
    )
fi

echo "[3/3] Prefetching raw HTTP assets used by build.rs"
mapfile -t urls < <(
    {
        rg -o --no-filename 'https://[[:alnum:]][[:alnum:].-]*/[^" )]+' build.rs
        rg -o --no-filename '^/\* !include https://[^ ]+' sqlpage/*.css sqlpage/*.js sqlpage/*.svg \
            | sed 's#^/\* !include ##'
    } | sort -u
)

for url in "${urls[@]}"; do
    file_name=$(printf '%s' "$url" | sed -E 's/[^[:alnum:].-]/_/g')
    cache_file="$cache_dir/$file_name"

    if [ -s "$cache_file" ]; then
        echo "  - Cached: $url"
        continue
    fi

    echo "  - Downloading: $url"
    curl -fsSL "$url" -o "$cache_file"
done

echo "Done. Offline caches are ready."
