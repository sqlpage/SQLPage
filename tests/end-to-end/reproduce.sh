#!/usr/bin/env bash
set -euo pipefail

usage="Usage: reproduce <fixture> <sqlite|postgres> <release tag or tree>..."
fixture=${1:?$usage}
database=${2:?$usage}
shift 2
if [ $# -eq 0 ]; then
  echo "$usage" >&2
  exit 2
fi

case $database in
  sqlite) ;;
  postgres)
    docker compose --file ../../docker-compose.yml up --wait postgres
    export FIXTURES_DATABASE_URL='postgres://root:Password123!@127.0.0.1/sqlpage'
    ;;
  *)
    echo "$usage" >&2
    exit 2
    ;;
esac

results=()
for version in "$@"; do
  release=$version
  if [ "$version" = tree ]; then
    release=
  fi
  if SQLPAGE_RELEASE=$release npx playwright test --config reproduce.config.ts "fixtures/$fixture" --reporter=line; then
    results+=("| $version | ✔ passed |")
  else
    results+=("| $version | ✘ failed |")
  fi
done

echo
echo "| $fixture | $database |"
echo "| --- | --- |"
printf '%s\n' "${results[@]}"
echo
echo "On commit $(git rev-parse --short HEAD): \`nix run .#reproduce -- $fixture $database $*\`"
