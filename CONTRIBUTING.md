# Contributing

## Setup

Prerequisites: the Rust toolchain pinned in `rust-toolchain.toml` (installed by `rustup`),
Python 3.11 or later, and [uv](https://docs.astral.sh/uv/). `make venv` creates `.venv` with
maturin and the test tools.

```bash
make help       # list every target
make venv       # create .venv with the dev tools
make py-dev     # build the extension into .venv (debug)
make gates      # format, clippy, Rust tests, benches compile, docs
make py-test    # build and run the Python test suite
make dist       # build the sdist and the release wheel into target/wheels
make smoke      # install the built wheel in a clean venv and run the suite
make stubs      # regenerate the native module's type stub (CI fails on drift)
make stubtest   # check the stub against the built module and the public API
```

## Test files

The files under `crates/oxedi_core/tests/fixtures/` and `crates/oxedi_core/tests/samples/` are
never edited.

## Golden files

```bash
UPDATE_GOLDEN=1 cargo test -p oxedi_core --test engine_golden    # regenerate, then inspect the diff
UPDATE_GOLDEN=1 cargo test -p oxedi_core --test project_golden   # table and diagnostic goldens
```

## DuckDB extension

From the repository root (Rust, Python 3 and the `extension-ci-tools` submodule are needed):

```bash
git submodule update --init
make configure_ci          # test venv, platform and version (the crate's, as v<version>)
make release               # build/release/extension/oxedi/oxedi.duckdb_extension
make test_release          # SQLLogicTest files in crates/oxedi_duckdb/test/sql
make duckdb-oracle         # read_835 against oxedi.parse_file, row by row
```

Load a local build with `duckdb -unsigned` and
`LOAD 'build/release/extension/oxedi/oxedi.duckdb_extension';`. `description.yml` in
`crates/oxedi_duckdb` is the descriptor for the community repository.

## Browser viewer

The viewer (`viewer/`) is a static page that runs the core as WebAssembly (`crates/oxedi_wasm`,
an internal binding with no public API). It has no JavaScript build step: `make wasm` builds the
module and generates `viewer/pkg/` (git-ignored).

```bash
make wasm              # build the module into viewer/pkg
make wasm-size-check   # fail if the gzipped module is over 250 KB
make viewer-serve      # build, then serve at http://127.0.0.1:8001
make viewer-test       # unit checks (node), the oracle against oxedi.parse, the browser tests
```

Tools: the `wasm32-unknown-unknown` Rust target (`make wasm` adds it), the `wasm-bindgen` CLI at
exactly the version `make wasm-bindgen-version` prints (from `Cargo.lock`), `wasm-opt` from
binaryen (CI uses release 133), Node 24 for the oracle and unit checks, and Chromium for the
browser tests. `crates/oxedi_wasm/scripts/install_tools.sh` installs the two prebuilt tools with
pinned checksums; CI and the deploy use it. Bumping the crate's `wasm-bindgen` pin means bumping
the version and checksum in that script. The browser tests run in the docs environment
(`make docs-venv`); install the browser once with `.venv-docs/bin/playwright install chromium`.

The page's Content-Security-Policy forbids network connections, so it loads the module from
generated ES modules instead of fetching it. Do not add a request, a font or a script from
another origin.

## Documentation

`README.md` is for GitHub and links to the guides under `docs/` with relative links.
`crates/oxedi_py/README.md` is the PyPI page and has the same content with absolute
`https://github.com/javillegasna/oxedi/blob/master/...` links, because relative links break on
PyPI. Change the root `README.md` and regenerate the other with `make readme-py`; `make gates` and CI
run `make readme-py-check`, which fails when the two differ.

## Documentation site

The site is built with MkDocs Material from `mkdocs.yml` and `docs/`. Its tools live in
`docs/requirements.txt`, in their own environment (`.venv-docs`), never in the package.

```bash
make docs         # strict build into site/: a broken link or a page outside the navigation fails
make docs-serve   # preview at http://127.0.0.1:8000
make release      # build the DuckDB extension once; the SQL examples load it
make docs-test    # run every python and sql block of docs/**/*.md and README.md
```

The API reference under `docs/reference/api/` is generated from the installed package and its type stub;
write a symbol's documentation in its docstring, not in the page.

`make docs-test` runs the blocks of a page in order, in one namespace (Python) or one DuckDB
connection (SQL), inside a temporary directory that holds `remittance.835`, `remittance-4010.835`, `big.835`, `a.835`,
`b.835`, `remits/*.835` and `remittances/*.835` (copies of the files under
`crates/oxedi_core/tests/fixtures/`). A block that cannot run carries a comment on the line
right before its fence, with a reason; the run lists every skipped block:

    <!-- docs-test: skip needs the reader's own tables -->

Use it for blocks that need something the runner cannot provide (a network, an orchestrator,
dbt templating), and when the equivalent runs elsewhere on the page, say so in the reason. A
`python` or `sql` block without the marker must run.

To add a page, put it in `docs/` and in the `nav` of `mkdocs.yml`; the strict build rejects a
page outside the navigation.

### Deploy

`.github/workflows/docs.yml` publishes with `mike` to the `gh-pages` branch. A push to `master`
that changes what the site or the viewer is built from (the `paths` of the workflow) publishes
`dev` (titled "dev (unreleased)"). A final tag `vX.Y.Z` publishes version `X.Y` and moves the `latest` alias, which
the site root redirects to. Pre-release tags and `duckdb-v*` tags publish nothing. Pull requests
only build the site (the Docs job in `ci.yml`). To publish a release that was tagged before the workflow existed, run "Docs deploy" manually
from `master` with the `version` input (`X.Y`, for example `0.5`); it also moves `latest`.
The same workflow publishes the viewer next to the versions: a push to `master` puts it in
`viewer/dev/`, and a final tag (or the manual run) puts it in `viewer/`, which keeps `viewer/dev/`.
It then checks that `versions.json`, every version directory and the viewer are still on
`gh-pages`. Never commit to `gh-pages` by hand.

## CI

`ci.yml` runs five areas, each a reusable workflow (`_rust.yml`, `_python.yml`, `_duckdb.yml`,
`_docs.yml`, `_viewer.yml`), only when files that area depends on changed: a pull request is
compared with its base, a push to `master` with the previous commit. The `Changes` job holds the
path filters; a change under `.github/` runs Rust, and a change to the composite actions in
`.github/actions/` runs every area. A draft pull request runs Rust only; the other areas start
when it is marked ready for review. A new push to a pull request cancels its running checks.

`nightly.yml` runs every area on `master` each night, whatever changed, plus the DuckDB
pre-release leg (`next`), which reports and does not block. Run it by hand from the Actions tab
when a change needs the full set before merging.

`CI result` is the one check to require: it fails when an area failed or was cancelled and
passes when each one passed or was not needed. Caches are written only from `master`;
pull requests read them. The checks of a push to `master` are never cancelled or replaced by
a later push.

## Releases

`make release-check TAG=v<version>` checks the tag, a clean tree and the changelog entry before
a release. The Python package and the DuckDB extension are versioned and tagged separately
(`v*` and `duckdb-v*`); see `make help` for the publishing targets.
