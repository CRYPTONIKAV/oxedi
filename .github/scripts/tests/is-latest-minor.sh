#!/usr/bin/env bash
# Checks is-latest-minor.sh against versions.json lists. Run: .github/scripts/tests/is-latest-minor.sh
set -uo pipefail

script="$(dirname "$0")/../is-latest-minor.sh"
failed=0

check() {
  local expected=$1 version=$2 json=$3 actual
  actual=$(printf '%s' "$json" | "$script" "$version")
  if [ "$actual" != "$expected" ]; then
    echo "FAIL: $version against $json: expected $expected, got $actual"
    failed=1
  fi
}

list='[{"version":"0.5","aliases":["latest"]},{"version":"0.4","aliases":[]},{"version":"dev","aliases":[]}]'
check false 0.4 "$list"
check true 0.5 "$list"
check true 0.6 "$list"
check true 1.0 "$list"
check true 0.10 '[{"version":"0.9","aliases":["latest"]}]'
check false 0.9 '[{"version":"0.10","aliases":["latest"]}]'
check true 0.1 '[]'
check true 0.1 '[{"version":"dev","aliases":[]}]'

if printf '[]' | "$script" 0.5.1 2>/dev/null; then
  echo "FAIL: 0.5.1 is not an X.Y version and must be refused"
  failed=1
fi

if [ "$failed" -eq 0 ]; then echo "is-latest-minor: ok"; fi
exit "$failed"
