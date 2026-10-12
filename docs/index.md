# oxedi

Read and write X12 EDI 835 files, the electronic remittance advice health plans send to
providers. oxedi turns each file into clean tables and writes tables back into a valid file. Its
core is written in Rust, and you use it from Python or from DuckDB.

- **Lossless.** Every byte of the file is kept. Nothing is dropped, and the file writes back
  exactly as it was read.
- **Ready-made tables.** Payments, claims, service lines and adjustments come out as typed
  tables for Polars, pandas, pyarrow or DuckDB.
- **Problems are data.** A file with questionable data still parses. Each problem is a
  diagnostic that names the rule, the position and the value.

[Get started](start/index.md){ .md-button .md-button--primary }

## Where to go next

<div class="grid cards" markdown>

- **[Get started](start/index.md)**

    Install, parse an example file, read its claims and one diagnostic.

- **[The 835 in five minutes](start/the-835.md)**

    What a remittance holds and how it maps to the tables.

- **[Python](python/reading.md)**

    Read files, stream large ones, write new ones and handle diagnostics.

- **[DuckDB](duckdb/index.md)**

    Query 835 files with `read_835` and write them with `COPY`, from any DuckDB client.

- **[Recipes](recipes/lakehouse.md)**

    A Parquet lakehouse, Polars analytics and pipeline steps.

- **[Reference](reference/tables.md)**

    Every table and column, a glossary of X12 terms and the Python API.

</div>

Coming from `edi-835-parser`? See [Migrating from edi-835-parser](migrating-from-edi-835-parser.md).
