#!/usr/bin/env bash
# Checks retry-gh-pages.sh against a local bare repository standing in for origin.
# Run: .github/scripts/tests/retry-gh-pages.sh
# The single-quoted bash -c bodies expand in the child shell.
# shellcheck disable=SC2016
set -uo pipefail

retry=$(cd "$(dirname "$0")/.." && pwd)/retry-gh-pages.sh
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
export RETRY_DELAY_UNIT=0 GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
failed=0

expect() {
  local what=$1 expected=$2 actual=$3
  if [ "$expected" != "$actual" ]; then
    echo "FAIL: $what: expected $expected, got $actual"
    failed=1
  fi
}

git init -q --bare "$tmp/origin.git"
git clone -q "$tmp/origin.git" "$tmp/work" 2>/dev/null
git clone -q "$tmp/origin.git" "$tmp/other" 2>/dev/null
# The work clone stays on another branch, as in the workflow, where gh-pages is only a branch
# mike updates; the push script switches to it for its commit.
git -C "$tmp/other" checkout -q -b gh-pages
git -C "$tmp/work" checkout -q -b gh-pages
git -C "$tmp/work" commit -q --allow-empty -m first
git -C "$tmp/work" push -q origin gh-pages
git -C "$tmp/work" checkout -q -b main
git -C "$tmp/other" fetch -q origin gh-pages
git -C "$tmp/other" reset -q --hard origin/gh-pages
cd "$tmp/work" || exit 1

# A command that commits on the local gh-pages and pushes it. With MOVE=1 another clone pushes
# first on every run until the counter reaches MOVE_UNTIL, so the push is rejected.
cat > "$tmp/push.sh" <<'S'
#!/usr/bin/env bash
set -eu
echo run >> "$COUNT"
runs=$(wc -l < "$COUNT")
if [ "$runs" -le "${MOVE_UNTIL:-0}" ]; then
  git -C "$OTHER" commit -q --allow-empty -m "other $runs"
  git -C "$OTHER" push -q origin HEAD:gh-pages
fi
git switch -q gh-pages
trap 'git switch -q main' EXIT
git commit -q --allow-empty -m "deploy $runs"
git push origin gh-pages
S
chmod +x "$tmp/push.sh"
export OTHER="$tmp/other"

runs_of() { if [ -f "$COUNT" ]; then wc -l < "$COUNT"; else echo 0; fi; }
new_count() { COUNT="$tmp/count.$1"; export COUNT; }

new_count success
"$retry" "$tmp/push.sh" > /dev/null 2>&1
expect "a push that works exits 0" 0 $?
expect "a push that works runs once" 1 "$(runs_of)"

new_count rejected_once
MOVE_UNTIL=1 "$retry" "$tmp/push.sh" > /dev/null 2>&1
expect "a rejected push then a good one exits 0" 0 $?
expect "a rejected push is redone once" 2 "$(runs_of)"
git fetch -q origin gh-pages
expect "the redone commit is on origin" "deploy 2" "$(git log -1 --format=%s origin/gh-pages)"

new_count always_rejected
MOVE_UNTIL=99 "$retry" "$tmp/push.sh" > /dev/null 2>&1
expect "a push rejected every time exits 1" 1 $?
expect "a push rejected every time runs 3 times" 3 "$(runs_of)"

new_count build_failure
"$retry" bash -c 'echo "mkdocs: build failed" >&2; exit 7' > /dev/null 2>&1
expect "a build failure keeps its status" 7 $?
new_count build_failure_runs
"$retry" bash -c 'echo run >> "$COUNT"; echo "build failed" >&2; exit 7' > /dev/null 2>&1
expect "a build failure runs once" 1 "$(runs_of)"

new_count remote_rejected
"$retry" bash -c 'echo run >> "$COUNT"; echo " ! [remote rejected] HEAD -> gh-pages (permission denied)" >&2; exit 1' > /dev/null 2>&1
expect "a push refused by the remote runs once" 1 "$(runs_of)"

new_count fetch_failure
git remote set-url origin "$tmp/missing.git"
"$retry" bash -c 'echo run >> "$COUNT"' > /dev/null 2>&1
expect "a fetch that keeps failing exits 1" 1 $?
expect "a fetch that keeps failing never runs the command" 0 "$(runs_of)"

if [ "$failed" -eq 0 ]; then echo "retry-gh-pages: ok"; fi
exit "$failed"
