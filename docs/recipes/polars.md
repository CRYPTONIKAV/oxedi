# Analytics with Polars

Read a folder of remittances into Polars frames and answer the usual questions: who denies,
which adjustments cost the most, how charges compare with payments.

## Load a folder

Tables join across a file by row number, and row numbers restart in each file, so add the file
name to every frame and join on it. Text columns are bytes in the tables; cast them to
`String` where you want to read or group by them.

```python
from pathlib import Path

import oxedi
import polars as pl

parts = {}
for path in sorted(Path("remits").glob("*.835")):
    result = oxedi.parse_file(path)
    for name in result.tables.keys():
        frame = result.tables[name].to_polars().with_columns(file=pl.lit(path.name))
        parts.setdefault(name, []).append(frame)

tables = {name: pl.concat(frames) for name, frames in parts.items()}

payments = tables["payments"].select(
    "file", payment=pl.col("row"), payer=pl.col("payer_name").cast(pl.String)
)
claims = tables["claims"].join(payments, on=["file", "payment"])
```

`claims` now has the payer on every row.

## Denials by payer

Claim status `4` is a denial. Count them next to the total.

```python
denials = claims.group_by("payer").agg(
    claims=pl.len(),
    denied=(pl.col("claim_status") == b"4").sum(),
    denied_charges=pl.col("charge_amount").filter(pl.col("claim_status") == b"4").sum(),
)
print(denials.with_columns(rate=pl.col("denied") / pl.col("claims")).sort("rate", descending=True))
```

## CO and PR adjustments

An adjustment belongs to a claim (when `service` is empty) or to one service line. Its
`group_code` says who bears the amount: `CO` contractual obligation, `PR` patient
responsibility, `OA` other adjustment. Join the claims to get the payer, then total by group
and reason.

```python
adjustments = tables["adjustments"].join(
    claims.select("file", "payer", claim=pl.col("row")), on=["file", "claim"]
)

by_group = (
    adjustments.with_columns(
        group=pl.col("group_code").cast(pl.String),
        reason=pl.col("reason_code").cast(pl.String),
    )
    .group_by("payer", "group", "reason")
    .agg(amount=pl.col("amount").sum(), adjustments=pl.len())
    .sort("amount", descending=True)
)
print(by_group.filter(pl.col("group").is_in(["CO", "PR"])))
```

## Charged versus paid

Money columns are exact decimals, so sums do not drift.

```python
summary = claims.group_by("payer").agg(
    charged=pl.col("charge_amount").sum(),
    paid=pl.col("payment_amount").sum(),
    patient=pl.col("patient_responsibility").sum(),
)
print(summary.with_columns(paid_share=(pl.col("paid") / pl.col("charged")).round(3)))
```

For large folders, use `oxedi.stream` per file to keep memory bounded; see
[Reading](../reading.md). When the data already lives in DuckDB or Parquet, see
[a Parquet lakehouse](lakehouse.md) and read the result with `pl.read_parquet`.
