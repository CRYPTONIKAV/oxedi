# DuckDB extension

The `oxedi` extension reads 835 files as DuckDB tables and writes tables back into an 835. It
uses the same parser, writer and tables as the Python package.

- Every DuckDB client gets it: Python, R, Java, Node, Go, .NET, Rust and the CLI.
- It is built on DuckDB's stable C API and loads on DuckDB 1.5.6 and later.

## Install

<!-- docs-test: skip the runner loads the locally built extension; INSTALL needs the network -->
```sql
INSTALL oxedi FROM community;
LOAD oxedi;
```

!!! warning "Writing needs extension 0.2.0"
    `INSTALL` gives extension 0.1.x from DuckDB's community repository, which reads files.
    Writing with `COPY` needs 0.2.0, which is not in the community repository yet. Until then,
    build the extension (see
    [CONTRIBUTING](https://github.com/javillegasna/oxedi/blob/master/CONTRIBUTING.md#duckdb-extension))
    and load the file.

## Read your first file

```sql
SELECT claim_id, claim_status, charge_amount, payment_amount
FROM read_835('remittance.835');
```

```text
┌──────────┬──────────────┬───────────────┬────────────────┐
│ claim_id │ claim_status │ charge_amount │ payment_amount │
│ varchar  │   varchar    │ decimal(38,2) │ decimal(38,2)  │
├──────────┼──────────────┼───────────────┼────────────────┤
│ CLM1001  │ 1            │        300.00 │         200.00 │
│ CLM1002  │ 2            │        700.00 │         400.00 │
│ CLM2001  │ 1            │        150.00 │         120.00 │
│ CLM2002  │ 4            │         80.00 │           0.00 │
└──────────┴──────────────┴───────────────┴────────────────┘
```

`read_835` returns the claims by default. Pick another table with `table_name`:

```sql
SELECT procedure_code, charge_amount, payment_amount, service_date
FROM read_835('remittance.835', table_name := 'services');
```

```text
┌────────────────┬───────────────┬────────────────┬──────────────┐
│ procedure_code │ charge_amount │ payment_amount │ service_date │
│    varchar     │ decimal(38,2) │ decimal(38,2)  │     date     │
├────────────────┼───────────────┼────────────────┼──────────────┤
│ 99213          │        200.00 │         150.00 │ 2024-01-05   │
│ 85025          │        100.00 │          50.00 │ 2024-01-05   │
│ 97110          │        700.00 │         450.00 │ 2024-01-06   │
│ 99212          │        150.00 │         150.00 │ 2024-01-07   │
└────────────────┴───────────────┴────────────────┴──────────────┘
```

[Tables and columns](../reference/tables.md) lists every table and column.

## Next

- [Reading with SQL](reading.md): parameters, folders, remote files, diagnostics.
- [Writing with COPY](writing.md){ #writing }: turn tables back into an 835.
- [Types and limits](limits.md): what `COPY` accepts, and the extension's versions.
