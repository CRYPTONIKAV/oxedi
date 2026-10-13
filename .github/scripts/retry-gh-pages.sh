#!/usr/bin/env bash
# Runs a command that pushes to gh-pages, up to 3 times. Before every attempt the local
# gh-pages branch (the one mike works on) is reset to origin/gh-pages, so a retry after a
# rejected push redoes its commit on top of what the other deploy published.
#
# Only two failures are retried: the fetch itself failing, and the command's output showing that
# the push was rejected because the remote moved (non-fast-forward, "fetch first"). Any other
# failure of the command, such as a broken site build, fails at once with the command's status.
# RETRY_DELAY_UNIT (seconds, default 5) scales the wait between attempts.
# Usage: retry-gh-pages.sh COMMAND [ARGS...]
set -uo pipefail

attempts=3
rejected='non-fast-forward|fetch first|\[rejected\]'
log=$(mktemp)
trap 'rm -f "$log"' EXIT

for attempt in $(seq 1 "$attempts"); do
  reason=""
  if ! git fetch origin gh-pages; then
    reason="git fetch origin gh-pages failed"
  else
    git branch -f gh-pages origin/gh-pages || exit 1
    "$@" 2>&1 | tee "$log"
    status=${PIPESTATUS[0]}
    if [ "$status" -eq 0 ]; then
      exit 0
    fi
    if ! grep -Eq "$rejected" "$log"; then
      echo "Not retrying: the failure is not a rejected push: $*" >&2
      exit "$status"
    fi
    reason="the push was rejected because gh-pages moved"
  fi
  echo "Attempt $attempt of $attempts failed ($reason): $*" >&2
  if [ "$attempt" -lt "$attempts" ]; then
    sleep $((attempt * ${RETRY_DELAY_UNIT:-5}))
  fi
done
echo "Giving up after $attempts attempts: $*" >&2
exit 1
