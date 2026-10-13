# Customizing the spec

A JSON spec defines the structure of the 835 and the columns of each table. You can choose a
version, or adapt the spec to a payer's variations with a patch, without touching the code.
`parse`, `parse_file` and `stream` all accept `spec=`.

## Choose a version

By default oxedi reads the version a file declares (`ISA12` and `GS08`) and uses the matching
built-in spec:

- 5010 (`005010X221A1`), the default.
- 4010 (`004010X091A1`), for 4010 files.

Pass `spec=` to override it, or pick one yourself:

```python
from pathlib import Path
import oxedi

data = Path("remittance-4010.835").read_bytes()
spec = oxedi.Spec.builtin(version="4010")
result = oxedi.parse(data, spec=spec)
print(result.spec, result.count_claims())
```

```text
Spec(name='835', loops=8, tables=5) 4
```

`result.spec` is the spec that projected the tables: the one you gave, or the built-in one
selected from the file's version.

## Add a column with a patch

A patch in JSON Merge Patch format (RFC 7386) adapts the spec. This one adds a column to
`claims`, read from `REF02` of the claim's `REF*CE`:

```python
spec = oxedi.Spec.builtin().patch({
    "tables": {"claims": {"columns": {
        "contract_class": {"segment": "REF", "where": {"1": "CE"}, "element": 2}
    }}}
})
result = oxedi.parse(Path("remittance.835").read_bytes(), spec=spec)
print("contract_class" in result.tables["claims"].columns)
```

```text
True
```

- Objects merge key by key.
- Arrays are replaced whole.
- `patch` accepts a dict or JSON text, and raises `oxedi.SpecError` when the result is not a
  valid spec.

## Choose where a column reads

A column reads one element of one segment. You pick the segment in one of two ways:

- `segment`, with optional `where` conditions on its elements:
  `{"segment": "REF", "where": {"1": "CE"}, "element": 2}`.
- A named occurrence of the loop: `{"occurrence": "patient_name", "element": 3}`.

Add `loop` to read a loop inside the table's anchor or above it. In the services table, this
column gives each service its claim's id:

```python
import polars as pl

spec = oxedi.Spec.builtin().patch({
    "tables": {"services": {"columns": {
        "claim_id": {"loop": "2100", "occurrence": "claim_payment_information", "element": 1}
    }}}
})
services = oxedi.parse_file("remittance.835", spec=spec).tables["services"].to_polars()
print(services.select("row", pl.col("claim_id", "procedure_code").cast(pl.String)))
```

```text
┌─────┬──────────┬────────────────┐
│ row ┆ claim_id ┆ procedure_code │
│ --- ┆ ---      ┆ ---            │
│ i64 ┆ str      ┆ str            │
╞═════╪══════════╪════════════════╡
│ 0   ┆ CLM1001  ┆ 99213          │
│ 1   ┆ CLM1001  ┆ 85025          │
│ 2   ┆ CLM1002  ┆ 97110          │
│ 3   ┆ CLM2001  ┆ 99212          │
└─────┴──────────┴────────────────┘
```

??? note "Details: when the segment repeats"
    When an occurrence repeats, `pick` chooses `"first"` (the default), `"last"` or the n-th
    match, counting from 1. It chooses among the segments read while the row's loop instance
    is open. A loop above the anchor offers the segments its enclosing instance read before the
    row's instance opened.

## Flag patient data

Which columns [hide patient data](../python/without-patient-data.md) when you pass a mask is
part of the spec. A column carries a `phi` key with one of three words:

- `"name"` and `"id"` become a keyed token. They need a text column (`AN` or `ID` element, or
  one the spec does not type).
- `"date"` shifts or reduces a date. It needs a date (`DT`) column.

A table can also carry `phi_seed`, the name of one `id` column. With `dates="shift"` its value
decides how many days the row and the rows under it move; the built-in `claims` table uses
`patient_id`.

```python
spec = oxedi.Spec.builtin().patch({
    "tables": {"claims": {"columns": {
        "contract_class": {"segment": "REF", "where": {"1": "CE"}, "element": 2, "phi": "id"}
    }}}
})
mask = oxedi.Mask("a-secret-of-at-least-16-bytes")
result = oxedi.parse(Path("remittance.835").read_bytes(), spec=spec, mask=mask)
print("contract_class" in spec.schema()["claims"].names)
```

```text
True
```

A patch that flags a numeric column, uses another word, flags a column that is not a date with
`"date"`, names a `phi_seed` column the table lacks, or names one that is not flagged `"id"`
raises `oxedi.SpecError` with the table, the column and the reason. A patch that removes or
unflags `claims.patient_id` must also set `"phi_seed": null` on `claims` (or name another `id`
column).

## Add or change a segment

A loop's segments are named occurrences. A patch adds one by its name, or changes it, or
removes it with `null`:

```json
{"loops": {"1000A": {"occurrences": {"xx": {"segment": "XX", "pos": 11400}}}}}
```

`pos` orders the occurrences of a loop. The occurrence a loop opens on comes first: every other
occurrence of the loop has a higher `pos`.

??? note "Details: how positions are numbered"
    The transaction and every loop below it share one position space. The built-in spec
    numbers a segment of the transaction's n-th table at n × 10000 plus its
    implementation-guide position: 1000A's N1 is 10800, 2100's CLP 20100. A child loop's
    occurrences sit at their own positions inside that space.

## Write with a custom spec

A patched spec also writes: pass it to `oxedi.write(frames, envelope, spec=spec)`. See
[Writing](../python/writing.md).
