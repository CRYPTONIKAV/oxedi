#!/usr/bin/env bash
# Prints "true" when the minor version X.Y given as the argument is the highest of those listed
# in a mike versions.json read from stdin, counting X.Y itself, and "false" otherwise. Entries
# that are not X.Y (such as dev) are ignored, and an empty list makes X.Y the highest.
# Usage: is-latest-minor.sh X.Y < versions.json
set -euo pipefail

version=${1:?usage: is-latest-minor.sh X.Y < versions.json}
if ! [[ "$version" =~ ^[0-9]+\.[0-9]+$ ]]; then
  echo "is-latest-minor.sh: expected X.Y, got $version" >&2
  exit 2
fi

highest=$({ jq -r '.[].version' | grep -E '^[0-9]+\.[0-9]+$' || true; echo "$version"; } | sort -V | tail -n 1)
if [ "$highest" = "$version" ]; then echo true; else echo false; fi
