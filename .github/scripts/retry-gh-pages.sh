#!/usr/bin/env bash
# Runs a command that pushes to gh-pages, up to 3 times. Before every attempt the local
# gh-pages branch (the one mike works on) is reset to origin/gh-pages, so a retry after a
# rejected non-fast-forward push redoes its commit on top of what the other deploy published.
# Usage: retry-gh-pages.sh COMMAND [ARGS...]
set -uo pipefail

attempts=3
for attempt in $(seq 1 "$attempts"); do
  git fetch origin gh-pages || exit 1
  git branch -f gh-pages origin/gh-pages || exit 1
  if "$@"; then
    exit 0
  fi
  echo "Attempt $attempt of $attempts failed: $*" >&2
  if [ "$attempt" -lt "$attempts" ]; then
    sleep $((attempt * 5))
  fi
done
exit 1
