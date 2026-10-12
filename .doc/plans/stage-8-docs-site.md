# Stage 8 · Documentation site — Implementation Plan

> Lean plan. Two batches of two tasks, one implementer per batch (Sonnet), Opus review per batch,
> Opus triage at the end, one fix wave.

**Goal:** a versioned MkDocs Material site on GitHub Pages built from `docs/`, with a Python API
reference generated from the stubs, every code example run in CI, three recipes, and the README as
a front page.

**Spec:** `.doc/architectural-commitment.md` §7 "Stage 8" (T94–T100), approved 2026-10-12.
Branch `stage-8-docs-site`. Ledger `.superpowers/sdd/stage-8-docs-site/progress.md`.

## Global Constraints
- Docs dependencies (`mkdocs-material`, `mkdocstrings[python]`, `mike`, the pytest plugin for
  Markdown blocks) live in a development group or a docs requirements file used by make targets
  and CI; the `oxedi` package's runtime and extras do not change.
- Everything user-facing in English; short, direct sentences; no implementation details (module
  paths of the binding, internal types, stage or principle codes).
- Every `python` and `sql` block in the site runs in CI (T97). A block that cannot run carries an
  explicit marker the runner understands; the runner reports the skipped count and each skip has a
  reason in the page source.
- Examples use conventional file names (`remittance.835`, `remits/*.835`, others as needed)
  provided by the runner from `crates/oxedi_core/tests/fixtures/` (fixtures are never edited).
- `mkdocs build --strict` passes with no warnings. Pages deploy only from tags `v*` (that minor's
  version, `latest` alias) and from `master` (`dev`); PRs build only.
- Gates on every commit: `make gates`; batches touching Python also `make py-test`; the new docs
  targets.

## Review Focus
1. Nothing lost from today's `docs/` and READMEs when pages move into the site navigation.
2. The example runner really executes every block (count blocks per page against the source) and
   fails on an error; skips are few and justified.
3. API reference completeness against the public API (`oxedi.__all__`, `oxedi.pyx12`,
   `oxedi.edi_835_parser`), no empty or private pages, docstrings rendered.
4. Deploy workflow: correct `mike` usage (version from the tag's minor, `latest` alias, `dev` from
   master), permissions minimal, no deploy from forks or PRs.
5. README front page and the generated PyPI README: links point to the versioned site, the drift
   check still passes.

---

## Batch A (Sonnet → Opus review): site and runner

### Task 1: site skeleton and API reference
- `mkdocs.yml` at the repo root: Material theme (light/dark), search, navigation: Home (from a
  short `docs/index.md`), Guides (reading, writing, DuckDB, spec, pyx12), Recipes (placeholder
  section for Task 3), API reference, Migrating from edi-835-parser, Contributing (link or page).
- API reference pages with `mkdocstrings` reading the installed package and its stubs
  (`_core.pyi`): one page per area (parsing and results, writing, spec, diagnostics, pyx12,
  edi_835_parser). Confirm griffe resolves the compiled module through the stubs; if a symbol
  renders empty, fix the docstring source in the binding rather than writing text by hand.
- Make targets: `docs` (strict build), `docs-serve` (local preview); the docs environment is
  created by a target (like `py-dev`), not by the package.
- CI: a `docs` job on PRs running the strict build (path filters like the other jobs).
- Commit: `docs: MkDocs site skeleton and Python API reference`.

### Task 2: example runner
- Python blocks: a pytest plugin for Markdown code blocks (choose between `pytest-markdown-docs`
  and `mktestdocs`; explain the choice) over `docs/**/*.md` and `README.md`.
- SQL blocks: a pytest test that extracts `sql` blocks and runs them in DuckDB (Python client,
  `allow_unsigned_extensions`) with the release extension build loaded.
- A fixture directory per test run with the conventional files; a skip marker syntax (e.g. an
  HTML comment before the block) with a reason; the run prints the skipped blocks.
- Make every current block run or mark it; fix examples that fail (stop and report if a failure is
  a library bug).
- Make target `docs-test`; CI runs it in the docs job (it needs the extension build for SQL: reuse
  the DuckDB job's build or build in the job).
- Commit: `test(docs): run every python and sql example`.

## Batch B (Sonnet → Opus review): recipes, deploy, front page

### Task 3: recipes
- `docs/recipes/`: `lakehouse.md` (DuckDB: folders and globs with `read_835`, Parquet partitioned
  by payer and month, `diagnostics` as a quality gate, dbt sources over `read_835`), `polars.md`
  (denials by payer, CO/PR adjustments, charged vs paid), `pipelines.md` (receive bytes, parse,
  check diagnostics, write out; short notes for Airflow, Dagster, Kafka). Every block runs (dbt
  itself is not installed: show its source YAML/SQL as non-executed blocks with the skip marker
  and run the equivalent SQL).
- Commit: `docs: recipes for a Parquet lakehouse, Polars analytics and pipelines`.

### Task 4: deploy, front page, metadata
- Deploy workflow with `mike`: on `v*` tags publish `X.Y` and move `latest`; on `master` publish
  `dev`; `gh-pages` branch; minimal permissions. The owner enables Pages (document the steps in
  the report).
- README as front page (80–100 lines: what it is, install, one read, one write, one DuckDB
  example, links to the site); regenerate the PyPI README (`make readme-py`), keeping the drift
  check; `Documentation` URL in `crates/oxedi_py/pyproject.toml` → the site.
- `CONTRIBUTING.md`: build and preview the site, the skip marker, the deploy flow.
- `CHANGELOG.md` `[Unreleased]`: the documentation site. `.doc/state.md`, `.doc/roadmap.md`.
- Commit: `docs: deploy the versioned site; README as front page`.

## Exit gate (§7 Stage 8)
- Strict build clean; every block run (skips listed with reasons); API reference complete.
- `dev` deployed and visible once the owner enables Pages; README and PyPI link to the site.
- `make gates`, `make py-test`, `make docs`, `make docs-test` green.
