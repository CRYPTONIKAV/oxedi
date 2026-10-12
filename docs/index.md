# oxedi

A fast, lossless parser and writer for X12 EDI 835 files (electronic remittance advice). The
core is written in Rust and is available from Python and from DuckDB.

- **Lossless.** Every byte of the file is kept. Nothing is silently dropped, and the file can
  be written back exactly as it was read.
- **Ready-made tables.** Payments, claims, service lines and adjustments come out as typed
  tables that open in Polars, pandas, pyarrow or DuckDB.
- **Problems are reported, not raised.** Questionable data still parses; each issue is a
  diagnostic that names the rule, the position and the value.

[Get started](start/index.md){ .md-button .md-button--primary }
[Install](start/install.md){ .md-button }

## Where to go next

| Page | What is in it |
|---|---|
| [Get started](start/index.md) | Install, parse a file and read its tables |
| [Reading](python/reading.md) | Tables in depth, the document; also [large files](python/large-files.md) and [diagnostics](python/diagnostics.md) |
| [Writing](python/writing.md) | The envelope, [your own tables](python/own-tables.md), [when the writer refuses](python/when-write-refuses.md) |
| [DuckDB](duckdb/index.md) | `read_835` parameters, `COPY` options, accepted types, limits |
| [The spec](customize/spec.md) | Spec versions and patches |
| [Validating with pyx12](python/pyx12.md) | What `validate` reports and where |
| [Migrating from edi-835-parser](migrating-from-edi-835-parser.md) | Change the import, or move to the native API |
| [Recipes](recipes/lakehouse.md) | A Parquet lakehouse, Polars analytics, pipelines |
| [Reference](reference/tables.md) | Tables and columns, glossary, the Python API |
