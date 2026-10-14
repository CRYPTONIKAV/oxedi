#!/usr/bin/env bash
# Publishes the site to gh-pages with mike and the browser viewer next to it, then checks what
# the run was meant to publish and fails naming what is missing.
#
#   dev        deploys the `dev` version and publishes the viewer into `viewer/dev/`.
#   release    deploys X.Y. When X.Y is the highest version on gh-pages, counting X.Y itself
#              (is-latest-minor.sh over versions.json), it also moves the `latest` alias and the
#              default onto X.Y, and publishes the viewer into `viewer/` and llms.txt at the root;
#              otherwise it deploys only its own folder and leaves `latest`, the viewer and
#              llms.txt alone.
#
# By default every read of gh-pages is of the commit a fresh `git fetch origin gh-pages`
# returned, and every push goes through retry-gh-pages.sh; gh-pages must already exist on
# origin, or the first fetch fails the run. With --no-push nothing is fetched or pushed: mike
# runs without --push, publish-viewer.sh commits onto the local gh-pages branch (PUSH=local),
# and the reads and the final checks are of that local branch, which may not exist yet.
#
# Runs from the repository root. Needs mike (and mkdocs) on the PATH, jq, a git identity, the
# viewer's module in viewer/pkg (make wasm) and GITHUB_SHA, which names the viewer's commit.
# Usage: deploy-docs.sh [--no-push] dev
#        deploy-docs.sh [--no-push] release X.Y
set -euo pipefail

usage() {
  echo "deploy-docs.sh: $1" >&2
  echo "usage: deploy-docs.sh [--no-push] dev | deploy-docs.sh [--no-push] release X.Y" >&2
  exit 2
}

scripts=$(cd "$(dirname "$0")" && pwd)
cd "$scripts/../.."

push=true
if [ "${1:-}" = "--no-push" ]; then
  push=false
  shift
fi
kind=${1:-}
version=""
case "$kind" in
  dev)
    [ $# -eq 1 ] || usage "dev takes no version, got: ${*:2}"
    ;;
  release)
    [ $# -eq 2 ] || usage "release takes exactly one version X.Y, got $(($# - 1)) arguments"
    version=$2
    [[ "$version" =~ ^[0-9]+\.[0-9]+$ ]] || usage "the release version must be X.Y, got $version"
    ;;
  *)
    usage "the kind must be dev or release, got '${kind}'"
    ;;
esac

if [ "$push" = true ]; then
  pages=""
  mike_push=(--push)
else
  pages=gh-pages
  mike_push=()
fi
# KIND and VERSION are read by publish-viewer.sh and by the inner shells below.
export NO_MKDOCS_2_WARNING=true KIND=$kind VERSION=$version

# Brings gh-pages up to date; a no-push run reads the local branch and fetches nothing. A push
# run reads the commit the fetch returned, not origin/gh-pages, which a single-branch or shallow
# clone may leave stale.
refresh() {
  if [ "$push" = true ]; then
    git fetch origin gh-pages
    pages=$(git rev-parse --verify --quiet 'FETCH_HEAD^{commit}') \
      || {
        echo "git fetch origin gh-pages did not produce a gh-pages commit in FETCH_HEAD" >&2
        exit 1
      }
  fi
}
# Runs a command that writes gh-pages: through retry-gh-pages.sh when it pushes, as is otherwise.
write_pages() {
  if [ "$push" = true ]; then "$scripts/retry-gh-pages.sh" "$@"; else "$@"; fi
}
on_pages() { git cat-file -e "$pages:$1" 2>/dev/null; }

# The viewers and llms.txt already published, which must still be there at the end.
refresh
release_before=false
dev_before=false
llms_before=false
if on_pages viewer/index.html; then release_before=true; fi
if on_pages viewer/dev/index.html; then dev_before=true; fi
if on_pages llms.txt; then llms_before=true; fi

moves=false
if [ "$kind" = "dev" ]; then
  write_pages mike deploy "${mike_push[@]}" --title "dev (unreleased)" dev
else
  refresh
  moves=$({ git show "$pages:versions.json" 2>/dev/null || echo '[]'; } | "$scripts/is-latest-minor.sh" "$version")
  echo "Version $version: latest moves = $moves"
  # The inner shells expand VERSION from the environment and "$@" from their own arguments
  # (--push, or nothing).
  # shellcheck disable=SC2016
  if [ "$moves" = "true" ]; then
    write_pages bash -ec '
      mike deploy "$@" --update-aliases "$VERSION" latest
      mike set-default "$@" latest' deploy "${mike_push[@]}"
  else
    write_pages bash -ec 'mike deploy "$@" "$VERSION"' deploy "${mike_push[@]}"
  fi
fi

# `viewer/` for a release that moves latest (keeping `viewer/dev/`), `viewer/dev/` for dev. A
# release also writes llms.txt at the root. Nothing else on gh-pages is touched.
if [ "$kind" = "dev" ] || [ "$moves" = "true" ]; then
  if [ "$push" = true ]; then
    write_pages "$scripts/publish-viewer.sh"
  else
    PUSH=local "$scripts/publish-viewer.sh"
  fi
fi

# What the run was meant to publish.
refresh
has() { on_pages "$1" || { echo "gh-pages lacks $1"; exit 1; }; }
has versions.json
git show "$pages:versions.json" | jq -r '.[].version' | while read -r listed; do
  has "$listed/index.html"
done
# An alias is a symlink to its version's folder.
git show "$pages:versions.json" | jq -r '.[].aliases[]' | while read -r alias; do
  has "$alias"
done
# A release deploy must have published its own folder and, when latest was meant to move, the
# alias on it.
if [ "$kind" = "release" ]; then
  has "$version/index.html"
  if [ "$moves" = "true" ]; then
    git show "$pages:versions.json" | jq -e --arg v "$version" 'any(.[]; .version == $v and (.aliases | index("latest")))' > /dev/null \
      || { echo "gh-pages versions.json does not list latest among the aliases of $version"; exit 1; }
  fi
fi
# A viewer that was there before the run must still be there (mike does not touch it); this
# run's own viewer must have been published.
if { [ "$kind" = "release" ] && [ "$moves" = "true" ]; } || [ "$release_before" = "true" ]; then has viewer/index.html; has viewer/pkg/index.js; fi
if { [ "$kind" = "release" ] && [ "$moves" = "true" ]; } || [ "$llms_before" = "true" ]; then has llms.txt; fi
if [ "$kind" = "dev" ] || [ "$dev_before" = "true" ]; then has viewer/dev/index.html; has viewer/dev/pkg/index.js; fi
for dir in viewer viewer/dev; do
  if on_pages "$dir/pkg/oxedi_bg.wasm"; then echo "gh-pages holds $dir/pkg/oxedi_bg.wasm, which the page never loads"; exit 1; fi
done
# The published page must carry the policy that forbids network connections.
for page in viewer/index.html viewer/dev/index.html; do
  if on_pages "$page"; then
    git show "$pages:$page" | grep -q "connect-src 'none'" || { echo "gh-pages $page lacks the Content-Security-Policy (connect-src 'none')"; exit 1; }
  fi
done
echo "gh-pages ($pages) holds what the $kind deploy${version:+ of $version} was meant to publish."
