#!/usr/bin/env bash
set -euo pipefail

if [[ "${PATHS_OUTCOME:-}" == success && "${ALL_EXAMPLES:-}" != true ]]; then
  examples="$MATCHED_EXAMPLES"
else
  # Tags, missing diff bases, and failed path filtering run every example.
  examples="$(find examples -mindepth 2 -maxdepth 2 -name test.hurl -print \
    | sed 's#/test.hurl$##' | sort | jq -R -s -c 'split("\n")[:-1]')"
fi

echo "examples=$examples" >> "$GITHUB_OUTPUT"
echo "Selected Hurl examples: $examples" >> "$GITHUB_STEP_SUMMARY"
