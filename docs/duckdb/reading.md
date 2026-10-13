# Reading with SQL

`read_835` reads 835 files into a table: one file, a folder, a list or a remote bucket. This
page covers its parameters, how it handles bad files and how it types text.

## Read one table of many files

`read_835(path, ...)` takes a path, a glob pattern or a list of them. It returns one table per
call:

```sql
FROM read_835('remittance.835');                                   -- claims (the default table)
FROM read_835('remits/*.835', table_name := 'services', filename := true);
FROM read_835(['a.835', 'b.835'], table_name := 'payments');
```

Add `filename := true` to know which file each row came from:

```sql
SELECT filename, count(*) AS services
FROM read_835('remits/*.835', table_name := 'services', filename := true)
GROUP BY filename
ORDER BY filename;
```

```text
┌──────────────┬──────────┐
│   filename   │ services │
│   varchar    │  int64   │
├──────────────┼──────────┤
│ remits/a.835 │        4 │
│ remits/b.835 │        4 │
│ remits/c.835 │       10 │
└──────────────┴──────────┘
```

!!! note "Joining tables across files"
    Row numbers restart in every file. Join a child table to its parent on `filename` and the
    row number together. [A Parquet lakehouse](../recipes/lakehouse.md) shows the join.

To save a table as Parquet, wrap the query in DuckDB's `COPY`:

```sql
COPY (SELECT * FROM read_835('remits/*.835', table_name := 'claims')) TO 'claims.parquet';
```

## Parameters

| Parameter | Default | Meaning |
|---|---|---|
| `table_name` | `'claims'` | `payments`, `claims`, `services`, `adjustments`, `provider_adjustments` or `diagnostics` |
| `filename` | `false` | add a `filename` column with the path each row came from |
| `version` | the file's own | force the built-in spec, `'5010'` or `'4010'`. By default each file uses the version it declares (5010 if none) |
| `binary` | `false` | return text columns as `BLOB` (the raw bytes) instead of `VARCHAR` |
| `ignore_errors` | `false` | report a file that is not an X12 interchange as a `diagnostics` row and go on |

!!! note "Forwarding a macro parameter"
    Inside a macro with a literal path, forward the parameter with `=`, as in
    `table_name = tbl`. DuckDB does not substitute a macro parameter given with `:=` or `=>`,
    so `read_835` refuses it when the macro is created.

    ```sql
    CREATE MACRO remit_table(tbl) AS TABLE
        SELECT * FROM read_835('remits/a.835', table_name = tbl);
    SELECT count(*) FROM remit_table('payments');
    ```

To see the columns of a table, describe it on any file:

```sql
DESCRIBE SELECT * FROM read_835('remittance.835', table_name := 'claims');
```

[Tables and columns](../reference/tables.md) lists them all. A schema that does not need a file
is tracked in [#138](https://github.com/javillegasna/oxedi/issues/138).

## Read the diagnostics

`table_name := 'diagnostics'` returns the findings of the parse, one row each:

```sql
SELECT filename, level, kind, datum
FROM read_835('remits/*.835', table_name := 'diagnostics', filename := true, ignore_errors := true);
```

The example files are clean, so this returns no rows.
[Diagnostics](../python/diagnostics.md) explains each field.

## Skip files that are not interchanges

Without `ignore_errors`, a file that is not an interchange fails the query. With it:

- The file contributes no rows to any table.
- It adds one `diagnostics` row whose `rule` is the error.

So run the query with `table_name := 'diagnostics'` as well, to see what was skipped. In that
row, `datum` holds the bytes found instead of `ISA`:

- By default, as escaped ASCII text: other bytes are written as `\xNN`, so it is always valid
  UTF-8.
- With `binary := true`, raw.

## Choose VARCHAR or BLOB for text

Text columns are `VARCHAR`. A field that is not valid UTF-8 fails the query, with an error that
names the file and the cell.

Pass `binary := true` to read those fields byte for byte. Nothing is lost to an encoding guess.

The other columns are typed as in the Python tables: decimals, dates and times.

## Read folders and remote files

- Glob patterns are expanded by a private in-memory DuckDB that honours your settings.
- `enable_external_access` and `disabled_filesystems` apply. With `enable_external_access` off,
  a glob pattern is an error.
- A remote pattern such as `s3://bucket/*.835` sees persistent secrets only, not temporary
  `CREATE SECRET` ones.
- Plain paths and lists are read through your own file system and secrets.
