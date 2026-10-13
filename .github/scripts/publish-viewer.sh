#!/usr/bin/env bash
# One attempt at publishing the viewer to gh-pages: builds a worktree from the current
# origin/gh-pages, replaces the viewer's folder, commits and pushes. Nothing else on gh-pages is
# touched. Run it through retry-gh-pages.sh so a rejected push is redone on a fresh worktree.
# Environment: KIND (release or dev), VERSION (release only), GITHUB_SHA, and PUSH=false to stop
# after the commit in the temporary worktree (a dry run that never pushes).
set -euo pipefail

git fetch origin gh-pages
pages=$(mktemp -d)/gh-pages
git worktree add -B viewer-publish "$pages" origin/gh-pages
trap 'git worktree remove --force "$pages"' EXIT

# `viewer/` for a release (keeping `viewer/dev/`), `viewer/dev/` for master.
if [ "$KIND" = "release" ]; then
  dest="$pages/viewer"
  mkdir -p "$dest"
  find "$dest" -mindepth 1 -maxdepth 1 ! -name dev -exec rm -rf {} +
  label="$VERSION"
else
  dest="$pages/viewer/dev"
  rm -rf "$dest"
  mkdir -p "$dest"
  label="dev"
fi
# The page embeds the module in pkg/module.js and never fetches the raw .wasm.
tar -C viewer --exclude=./tests --exclude=./pkg/oxedi_bg.wasm -cf - . | tar -C "$dest" -xf -
cd "$pages"
git add -A viewer
if git diff --cached --quiet; then
  echo "The viewer on gh-pages is already up to date."
else
  git commit -m "Viewer $label from ${GITHUB_SHA::7}"
  if [ "${PUSH:-true}" = "true" ]; then
    git push origin HEAD:gh-pages
  else
    echo "PUSH=false: committed $(git rev-parse --short HEAD) locally, nothing pushed."
  fi
fi
