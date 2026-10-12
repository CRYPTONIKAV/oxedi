# oxedi

A fast, lossless parser and writer for X12 EDI 835 files (electronic remittance advice). The
core is written in Rust and is available from Python and from DuckDB.

- **Lossless.** Every byte of the file is kept. Nothing is silently dropped, and the file can
  be written back exactly as it was read.
- **Ready-made tables.** Payments, claims, service lines and adjustments come out as typed
  tables that open in Polars, pandas, pyarrow or DuckDB.
- **Problems are reported, not raised.** Questionable data still parses; each issue is a
  diagnostic that names the rule, the position and the value.

## Install

```bash
pip install oxedi
```

Requires Python 3.11 or later. For DuckDB, see [DuckDB](duckdb.md).

## Where to go next

| Page | What is in it |
|---|---|
| [Reading](reading.md) | Tables in depth, large files, diagnostics, the document |
| [Writing](writing.md) | The envelope, findings, balancing rules, writing from your own tables |
| [DuckDB](duckdb.md) | `read_835` parameters, `COPY` options, accepted types, limits |
| [The spec](spec.md) | Spec versions and patches |
| [Validating with pyx12](pyx12.md) | What `validate` reports and where |
| [Migrating from edi-835-parser](migrating-from-edi-835-parser.md) | Change the import, or move to the native API |
| [API reference](api/parsing.md) | Every public function and class of the Python package |
