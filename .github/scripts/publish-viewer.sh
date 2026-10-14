#!/usr/bin/env bash
# One attempt at publishing the viewer (and, for a release, the site's llms.txt) to gh-pages:
# builds a worktree from the gh-pages commit it just fetched, replaces the viewer's folder, commits and
# pushes. Nothing outside viewer/ and, for a release, the root llms.txt is touched. Run it
# through retry-gh-pages.sh so a rejected push is redone on a fresh worktree.
# Environment: KIND (release or dev), VERSION (release only), GITHUB_SHA, and PUSH: true (the
# default) pushes; false stops after the commit in the temporary worktree (a dry run that never
# pushes); local starts from the local gh-pages branch instead of the fetched one, fetches
# nothing, and moves that branch onto the commit (deploy-docs.sh --no-push).
set -euo pipefail

push=${PUSH:-true}
if [ "$push" = "local" ]; then
  base=gh-pages
else
  git fetch origin gh-pages
  base=FETCH_HEAD
fi
pages=$(mktemp -d)/gh-pages
git worktree add -B viewer-publish "$pages" "$base"
trap 'git worktree remove --force "$pages"' EXIT

# `viewer/` for a release (keeping `viewer/dev/`), `viewer/dev/` for master.
if [ "$KIND" = "release" ]; then
  dest="$pages/viewer"
  mkdir -p "$dest"
  find "$dest" -mindepth 1 -maxdepth 1 ! -name dev -exec rm -rf {} +
  label="$VERSION"
  # llms.txt at the site root follows `latest`, like the viewer; mike only copies it into each
  # version's folder.
  cp docs/llms.txt "$pages/llms.txt"
  paths="viewer llms.txt"
  what="Viewer and llms.txt"
else
  dest="$pages/viewer/dev"
  rm -rf "$dest"
  mkdir -p "$dest"
  label="dev"
  paths="viewer"
  what="Viewer"
fi
# The page embeds the module in pkg/module.js and never fetches the raw .wasm.
tar -C viewer --exclude=./tests --exclude=./pkg/oxedi_bg.wasm -cf - . | tar -C "$dest" -xf -
cd "$pages"
# shellcheck disable=SC2086
git add -A $paths
if git diff --cached --quiet; then
  echo "$what on gh-pages: already up to date."
else
  git commit -m "$what $label from ${GITHUB_SHA::7}"
  if [ "$push" = "true" ]; then
    git push origin HEAD:gh-pages
  elif [ "$push" = "local" ]; then
    git branch -f gh-pages HEAD
    echo "PUSH=local: moved the local gh-pages to $(git rev-parse --short HEAD), nothing pushed."
  else
    echo "PUSH=false: committed $(git rev-parse --short HEAD) locally, nothing pushed."
  fi
fi
