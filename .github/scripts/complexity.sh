#!/usr/bin/env bash
# Measures cyclomatic complexity and nesting of every function in the Rust, Python and
# JavaScript sources and compares them with .github/complexity-baseline.txt (complexity.py says
# how). Tests are not measured, and only files git tracks are, so target/, site/, viewer/pkg/
# and the generated type stubs (.pyi) never are.
# Needs uv: uvx runs complexity.py with the pinned lizard. Arguments go to complexity.py;
# --update rewrites the baseline once no function got worse.
set -euo pipefail

LIZARD_VERSION="1.24.1"
PYGMENTS_VERSION="2.21.0"
PATHSPEC_VERSION="1.1.1"
PYTHON_VERSION="3.13"

cd "$(dirname "$0")/../.."
files=$(mktemp)
trap 'rm -f "$files"' EXIT

git ls-files -- \
  'crates/*/src/*.rs' 'crates/*/python/*.py' 'crates/*/scripts/*.py' 'viewer/js/*.js' \
  | grep -vE '(^|/)tests?(/|\.rs$)' > "$files"

uvx --python "$PYTHON_VERSION" --from "lizard==$LIZARD_VERSION" \
  --with "pygments==$PYGMENTS_VERSION" --with "pathspec==$PATHSPEC_VERSION" \
  python .github/scripts/complexity.py .github/complexity-baseline.txt --files "$files" "$@"
