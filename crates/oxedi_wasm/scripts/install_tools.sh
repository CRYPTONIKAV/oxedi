#!/usr/bin/env bash
# Installs the prebuilt tools `make wasm` needs, into ~/.local/bin, and puts that on the PATH of
# the following GitHub Actions steps. Each download is checked against a pinned SHA-256.
#
# The wasm-bindgen CLI must be the version Cargo.lock resolves for oxedi_wasm. Bumping the
# crate's wasm-bindgen pin means bumping WASM_BINDGEN_VERSION and WASM_BINDGEN_SHA256 here; the
# script fails while they disagree with Cargo.lock. Used by the Viewer job in ci.yml and by the
# viewer step of docs.yml.
set -euo pipefail

WASM_BINDGEN_VERSION="0.2.129"
WASM_BINDGEN_SHA256="82d12bb940e2d4e72e0d5605387fc1b8ca179044e012b620f0ce4e7440e8320e"
# binaryen release that provides wasm-opt.
BINARYEN_VERSION="133"
BINARYEN_SHA256="2dc9c7813f5375db93d96ead4b78222fcc3e2677bbb832297af4797782a37489"

root=$(cd "$(dirname "$0")/../../.." && pwd)
locked=$(make --no-print-directory -C "$root" wasm-bindgen-version)
if [ "$locked" != "$WASM_BINDGEN_VERSION" ]; then
  echo "Cargo.lock resolves wasm-bindgen $locked but $0 pins $WASM_BINDGEN_VERSION; update WASM_BINDGEN_VERSION and WASM_BINDGEN_SHA256" >&2
  exit 1
fi

bin="${WASM_TOOLS_BIN:-$HOME/.local/bin}"
mkdir -p "$bin"
if [ -n "${GITHUB_PATH:-}" ]; then echo "$bin" >> "$GITHUB_PATH"; fi
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"

# Downloads URL and fails, naming the URL and both hashes, unless it has the pinned SHA-256.
fetch() {
  local url=$1 expected=$2 file actual
  file=$(basename "$url")
  curl -fsSLO "$url"
  actual=$(sha256sum "$file" | cut -d' ' -f1)
  if [ "$actual" != "$expected" ]; then
    echo "checksum mismatch for $url: expected $expected, got $actual" >&2
    exit 1
  fi
}

name="wasm-bindgen-$WASM_BINDGEN_VERSION-x86_64-unknown-linux-musl"
fetch "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/$WASM_BINDGEN_VERSION/$name.tar.gz" "$WASM_BINDGEN_SHA256"
tar xzf "$name.tar.gz"
install "$name/wasm-bindgen" "$bin/"

name="binaryen-version_$BINARYEN_VERSION-x86_64-linux"
fetch "https://github.com/WebAssembly/binaryen/releases/download/version_$BINARYEN_VERSION/$name.tar.gz" "$BINARYEN_SHA256"
tar xzf "$name.tar.gz"
install "binaryen-version_$BINARYEN_VERSION/bin/wasm-opt" "$bin/"
