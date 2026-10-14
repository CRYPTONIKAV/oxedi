#!/usr/bin/env bash
# Checks complexity.py's comparison with a baseline on hand-written measurements (no lizard).
# Run: .github/scripts/tests/complexity.sh
set -uo pipefail

script="$(dirname "$0")/../complexity.py"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
failed=0

baseline() {
  printf '# header\n' > "$tmp/baseline"
  printf '%s\n' "$@" | tr ' ' '\t' >> "$tmp/baseline"
}

measurements() {
  printf '%s\n' "$@" > "$tmp/measured.csv"
}

# expect STATUS PATTERN [ARGS...]: runs the check and wants that exit status and output line.
expect() {
  local status=$1 pattern=$2 output actual
  shift 2
  output=$(python3 "$script" "$tmp/baseline" --measurements "$tmp/measured.csv" "$@" 2>&1)
  actual=$?
  if [ "$actual" != "$status" ] || ! grep -qE "$pattern" <<< "$output"; then
    echo "FAIL: wanted status $status and /$pattern/, got $actual:"
    echo "$output"
    failed=1
  fi
}

baseline 'a.rs big 12 2' 'a.rs fmt 20 0' 'a.rs fmt 9 1'
measurements 'a.rs,big,12,2' 'a.rs,fmt,9,1' 'a.rs,fmt,20,0' 'a.rs,small,7,3'
expect 0 'ok, 4 functions measured'

measurements 'a.rs,big,12,2' 'a.rs,fmt,9,1' 'a.rs,fmt,20,0' 'a.rs,small,8,3'
expect 1 'FAIL a.rs: small: CCN 8, nesting 3 is over the targets'

measurements 'a.rs,big,12,2' 'a.rs,fmt,9,1' 'a.rs,fmt,20,0' 'a.rs,small,2,4'
expect 1 'FAIL a.rs: small: CCN 2, nesting 4 is over the targets'

measurements 'a.rs,big,13,2' 'a.rs,fmt,9,1' 'a.rs,fmt,20,0'
expect 1 "FAIL a.rs: big: CCN 13, nesting 2 is worse than the baseline's CCN 12, nesting 2"

measurements 'a.rs,big,12,4' 'a.rs,fmt,9,1' 'a.rs,fmt,20,0'
expect 1 "FAIL a.rs: big: CCN 12, nesting 4 is worse"

# Nesting within the target is not a regression of a function kept for its CCN.
measurements 'a.rs,big,12,3' 'a.rs,fmt,9,1' 'a.rs,fmt,20,0'
expect 0 'ok'

measurements 'a.rs,big,11,2' 'a.rs,fmt,9,1' 'a.rs,fmt,20,0'
expect 1 "STALE a.rs: big: CCN 11, nesting 2 is better than the baseline's CCN 12, nesting 2"

measurements 'a.rs,big,7,2' 'a.rs,fmt,9,1' 'a.rs,fmt,20,0'
expect 1 'STALE a.rs: big: baseline entry CCN 12, nesting 2 is now within the targets'

measurements 'a.rs,fmt,9,1' 'a.rs,fmt,20,0'
expect 1 'STALE a.rs: big: baseline entry .* or the function is gone'

# Functions sharing a name are compared worst with worst: a third one over the target is new.
measurements 'a.rs,big,12,2' 'a.rs,fmt,9,1' 'a.rs,fmt,20,0' 'a.rs,fmt,8,0'
expect 1 'FAIL a.rs: fmt: CCN 8, nesting 0 is over the targets'

measurements 'a.rs,big,13,2' 'a.rs,fmt,9,1' 'a.rs,fmt,20,0'
expect 1 'baseline was not updated' --update
grep -q 'big	12	2' "$tmp/baseline" || { echo "FAIL: a refused update changed the baseline"; failed=1; }

measurements 'a.rs,big,10,2' 'a.rs,fmt,20,0'
expect 0 'wrote .*rs 2' --update
if ! grep -q 'big	10	2' "$tmp/baseline" || grep -q 'fmt	9' "$tmp/baseline"; then
  echo "FAIL: the update did not lower big and drop the second fmt:"
  cat "$tmp/baseline"
  failed=1
fi

baseline 'a.rs big twelve 2'
expect 1 'expected path, function, ccn and nesting separated by tabs'

if [ "$failed" = 0 ]; then echo "complexity: ok"; fi
exit "$failed"
