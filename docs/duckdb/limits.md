# Types and limits

Which column types `COPY` accepts, what it refuses, and how the extension is versioned.

## Types COPY accepts

Per column of the spec:

| Column type | Accepts |
|---|---|
| Text | `VARCHAR`, `ENUM` or `BLOB` |
| Integer | Any integer type, or a `FLOAT` or `DOUBLE` holding a whole number |
| Decimal | Any `DECIMAL` or integer type that holds the value exactly. Never `FLOAT` or `DOUBLE`. |
| Date | `DATE`, or `TIMESTAMP` at midnight |
| Time | `TIME` or `TIME_NS` in whole seconds |

[Tables and columns](../reference/tables.md) gives the type of every column.

!!! warning "A bare `NULL` inside a struct"
    DuckDB types a bare `NULL` inside a struct as `INTEGER`, and a text column refuses it.
    Write `NULL::VARCHAR`, or the column's type.

## What COPY refuses

- `PARTITION_BY` and `PER_THREAD_OUTPUT` ask for several files. DuckDB consumes them, and the
  format refuses a second file.
- `compression` is refused too. Compress the file afterwards.

## What happens to an existing file

- With DuckDB's default temporary file and a local path, an existing file is left untouched
  when the `COPY` fails.
- Without it (`USE_TMP_FILE false`, or a path that is not local), the file is written over from
  its first byte.
- In that case, an existing file longer than the new content is refused ("remove it first").
- What happens to the target of a failed `COPY` is then up to DuckDB. Some versions remove it.

## Versions

- The extension is versioned separately from the Python package. Its version is in the crate's
  `Cargo.toml`.
- Its releases are tagged `duckdb-v*`.
- Its own changelog is
  [`crates/oxedi_duckdb/CHANGELOG.md`](https://github.com/javillegasna/oxedi/blob/master/crates/oxedi_duckdb/CHANGELOG.md).
