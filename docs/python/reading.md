# Reading 835 files

How to read a file into tables and get the document back byte for byte. For large files see
[Large files](large-files.md), for findings see [Diagnostics](diagnostics.md).

## Parse

```python
import oxedi

result = oxedi.parse_file("remittance.835")   # or oxedi.parse(data) from bytes

print(result.count_claims(), result.sum_payments())  # sum_payments() is a Decimal
print(result.payer["name"], result.payee["name"])
```

`result.payer` and `result.payee` are dicts of text values (`name`, `identification_code`,
`address`, `city`, `state`, `zip_code`). They are `None` when the file has no such party, and
raise `ValueError` when `payments` has more than one row.

A file that is not an 835 at all (for example, one that does not start with an `ISA`
segment) raises `oxedi.ParseError`. Anything else that parses is returned, with its problems
as diagnostics.

`parse`, `parse_file` and `stream` read the version a file declares and use the matching
built-in spec. See [Spec](../customize/spec.md) to choose another one or to extend it.

## Tables

| Table | One row per | Linked by |
|---|---|---|
| `payments` | payment (transaction) | none |
| `claims` | claim | `payment` |
| `services` | service line | `payment`, `claim` |
| `adjustments` | claim or service adjustment | `payment`, `claim`, `service` |
| `provider_adjustments` | provider-level adjustment | `payment` |

`result.tables.keys()` lists them and `table.columns` lists a table's columns. Amounts are
decimals, dates are dates, and text fields are kept as the raw bytes from the file (Arrow
`binary`), so nothing is lost to an encoding guess.

Each table converts to the library you use:

```python
claims = result.tables["claims"].to_polars()      # or .to_pandas()
```

Tables follow the Arrow PyCapsule interface, so any Arrow-aware library reads them directly,
without copying the data:

```python
import duckdb, polars as pl

claims = result.tables["claims"]
duckdb.sql("select claim_status, sum(payment_amount) from claims group by 1")
pl.DataFrame(result.tables["services"])
```

`row` counts the rows of a table from 0. `payment`, `claim` and `service` hold the `row` of
the parent, so a service joins to its claim on `services.claim = claims.row`.

## The document and writing the file back

Nothing is dropped: every byte of the file belongs to exactly one segment. The document keeps
them, so it returns the file exactly as it was read:

```python
from pathlib import Path

data = Path("remittance.835").read_bytes()
assert oxedi.parse(data).document.write() == data
```

A file that starts with a UTF-8 byte order mark is still read: the mark stays in the first
segment, `write()` gives the file back unchanged, and a `ByteOrderMark` diagnostic says it was
there.

`document.write()` reproduces the file. To produce a new file from tables, see
[Writing](writing.md).
