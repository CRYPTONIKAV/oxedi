# Writing 835 files

`oxedi.write` turns tables back into an 835: the tables of a parse, or your own Arrow, Polars
or pandas tables with the spec's columns. To reproduce a parsed file byte for byte, use
`result.document.write()` instead (see [Reading](reading.md#the-document-and-writing-the-file-back)).

## Parse, change, write

```python
import datetime
import polars as pl
import oxedi

result = oxedi.parse_file("remittance.835")
frames = {name: result.tables[name].to_polars() for name in result.tables.keys()}
frames["payments"] = frames["payments"].with_columns(trace_number=pl.lit("CHK100235"))

envelope = oxedi.Envelope(
    sender_id="ACMEPAYER",
    receiver_id="SUNRISECLINIC",
    date=datetime.date(2024, 1, 10),
    time=datetime.time(9, 0),
    usage_indicator="T",  # "P" (production) is the default
)
data = oxedi.write(frames, envelope, spec=result.spec)  # bytes, one interchange
```

## Which spec writes

The spec is the one that parsed the tables, else the built-in 5010 spec. A dictionary of
frames, as above, does not carry the parse's spec, so pass it: `spec=result.spec`
(`result.tables.spec` is the same one). For a 4010 file you can also pass
`oxedi.Spec.builtin("4010")`. Without it a 4010 file is written as 5010 and refused.

## What is written

A table left out has no rows and a column left out is null. Rows nest by their `payment`,
`claim` and `service` columns, the row number of their parent.

Only what the tables hold is written: segments no column reads (for example the payer's
`PER*CX` or the bank details of `BPR`) are left out. The transaction set header carries no
implementation convention reference, so a written 5010 header is `ST*835*0001~`.

## Envelope

`oxedi.Envelope` gives what the tables cannot:

- `sender_id` and `receiver_id`, with their qualifiers `sender_qualifier` and
  `receiver_qualifier` (default `"ZZ"`).
- `date` and `time`.
- `usage_indicator` (`"P"` by default).
- `control_number`, the first one (default 1).
- `application_sender` and `application_receiver`, optional, for the group header.
- `delimiters`: default `*`, `:`, `~` and repetition `^`.
- `line_break`: a line break after each segment.

The writer works out the rest: the fixed-width `ISA`, the control numbers that must match
(`ISA13`/`IEA02`, `GS06`/`GE02`, `ST02`/`SE02`) and the counts (`SE01`, `GE01`, `IEA01`).

## From DuckDB

The `oxedi` extension writes the same file from SQL, byte for byte equal to `oxedi.write` with
the same envelope. See [DuckDB](../duckdb/writing.md).

## Next

- [Writing from your own tables](own-tables.md)
- [When the writer refuses a file](when-write-refuses.md): findings, money and empty text.
