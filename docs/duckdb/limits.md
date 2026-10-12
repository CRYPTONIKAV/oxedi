# Types and limits

Which column types `COPY` accepts, what it refuses, and which versions the extension has.

## Accepted types

Per column of the spec:

- Text takes `VARCHAR`, `ENUM` or `BLOB`.
- An integer column takes any integer type, or a `FLOAT` or `DOUBLE` holding a whole number.
- A decimal column takes any `DECIMAL` or integer type that holds the value exactly (never
  `FLOAT` or `DOUBLE`).
- A date column takes `DATE`, or `TIMESTAMP` at midnight.
- A time column takes `TIME` or `TIME_NS` in whole seconds.

## Limits

- `PARTITION_BY` and `PER_THREAD_OUTPUT` ask for several files; DuckDB consumes them and the
  format refuses a second file. `compression` is refused too: compress afterwards.
- An existing file is left untouched when the `COPY` fails only for a local path written with
  DuckDB's default temporary file. Without it (`USE_TMP_FILE false`, or a path that is not
  local) the file is written over from its first byte, an existing file longer than the new
  content is refused ("remove it first") and what happens to the target of a failed `COPY` is
  up to DuckDB (some versions remove it).
- A bare `NULL` inside a struct is typed `INTEGER` by DuckDB and a text column refuses it;
  write `NULL::VARCHAR` (or the column's type).

## Versions

The extension is versioned separately from the Python package (its version is in the crate's
`Cargo.toml`), and its releases are tagged `duckdb-v*`. Its own changelog is
[`crates/oxedi_duckdb/CHANGELOG.md`](https://github.com/javillegasna/oxedi/blob/master/crates/oxedi_duckdb/CHANGELOG.md).
