# DuckDB extension

The `oxedi` extension reads X12 EDI 835 remittance files as DuckDB tables, and writes tables
back into an 835, with the same parser, writer and tables as the Python package. Every DuckDB
client (Python, R, Java, Node, Go, .NET, Rust, the CLI) gets it. It is built on DuckDB's stable
C API and loads on DuckDB 1.5.6 and later.

## Install

<!-- docs-test: skip the runner loads the locally built extension; INSTALL needs the network -->
```sql
INSTALL oxedi FROM community;
LOAD oxedi;
```

`INSTALL` gives extension 0.1.x from DuckDB's community repository, which reads files. Writing
with `COPY` needs 0.2.0, which is not in the community repository yet. Until then, build the
extension (see
[CONTRIBUTING](https://github.com/javillegasna/oxedi/blob/master/CONTRIBUTING.md#duckdb-extension))
and load the file.

Next: [Reading with SQL](reading.md), [Writing with COPY](writing.md), [Types and limits](limits.md).
