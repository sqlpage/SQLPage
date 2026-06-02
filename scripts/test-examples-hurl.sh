#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FILTER="${1:-}"

if ! command -v hurl >/dev/null 2>&1; then
  echo "hurl is required. See https://hurl.dev/docs/installation.html" >&2
  exit 1
fi

current_project=""
current_compose=""

cleanup() {
  if [[ -n "$current_project" && -n "$current_compose" ]]; then
    docker compose -p "$current_project" -f "$current_compose" down --volumes --remove-orphans >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

run_index=0
tested=0
while IFS= read -r -d "" test_file; do
  dir="$(dirname "$test_file")"
  rel_dir="${dir#"$ROOT_DIR/"}"

  if [[ -n "$FILTER" ]]; then
    if [[ "$FILTER" == */* ]]; then
      [[ "$rel_dir" == "$FILTER" ]] || continue
    else
      [[ "$rel_dir" == *"$FILTER"* ]] || continue
    fi
  fi

  run_index=$((run_index + 1))
  tested=$((tested + 1))
  current_project="sqlpage_example_${run_index}"
  current_compose="$dir/docker-compose.yml"

  echo "::group::Testing $rel_dir"
  if ! docker compose -p "$current_project" -f "$current_compose" up -d --quiet-pull --build --remove-orphans; then
    echo "::error file=$rel_dir/docker-compose.yml,title=Example compose startup failed::$rel_dir failed to start"
    echo "::group::docker compose ps for $rel_dir"
    docker compose -p "$current_project" -f "$current_compose" ps -a
    echo "::endgroup::"
    echo "::group::docker compose logs for $rel_dir"
    docker compose -p "$current_project" -f "$current_compose" logs
    echo "::endgroup::"
    echo "::endgroup::"
    exit 1
  fi
  if ! hurl --test \
    --retry 60 \
    --retry-interval 1s \
    --connect-timeout 2s \
    --error-format long \
    "$test_file"; then
    echo "::error file=$rel_dir/test.hurl,title=Hurl example test failed::$rel_dir failed"
    echo "::group::docker compose ps for $rel_dir"
    docker compose -p "$current_project" -f "$current_compose" ps
    echo "::endgroup::"
    echo "::group::docker compose logs for $rel_dir"
    docker compose -p "$current_project" -f "$current_compose" logs
    echo "::endgroup::"
    echo "::endgroup::"
    exit 1
  fi
  docker compose -p "$current_project" -f "$current_compose" down --volumes --remove-orphans
  echo "::endgroup::"

  current_project=""
  current_compose=""
done < <(find "$ROOT_DIR/examples" -mindepth 2 -maxdepth 2 -name test.hurl -print0 | sort -z)

if [[ "$tested" -eq 0 ]]; then
  echo "No examples matched filter: $FILTER" >&2
  exit 1
fi
