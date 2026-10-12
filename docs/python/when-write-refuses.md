# When the writer refuses a file

What the strict writer reports, what it never changes, and how null and empty text are written.
The examples reuse the tables and envelope of [Writing 835 files](writing.md).

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
    usage_indicator="T",
)
```

## Findings

The writer is strict by default. The written file is read back with the spec, and every
diagnostic of that read is a finding, as is anything the writer cannot place: for example a
required element or occurrence without a value, a value outside its code list or length, a row
whose parent does not exist or that comes out of order, a value holding a delimiter, or money
that does not balance. A value holding a delimiter is reported first: the file then splits
where the data does not, so it is not read back, and the other findings show once that value is
fixed.

Any finding raises `oxedi.WriteError`, a `ValueError` whose message lists every finding and
whose `findings` holds them as `oxedi.WriteFinding` (each names the table, row and column, or
the envelope field, with the diagnostic behind it). Nothing is written. With
`allow_findings=True` the call returns `(data, findings)` instead, for example to produce
invalid files for tests:

```python
try:
    data = oxedi.write(frames, envelope, spec=result.spec)
except oxedi.WriteError as error:
    for finding in error.findings:
        print(finding.table, finding.row, finding.column, finding)

data, findings = oxedi.write(frames, envelope, spec=result.spec, allow_findings=True)
```

## Money

The writer never changes money. The balancing rules are checked, never used to fill a total:

- `BPR02` against the claims' payments and the `PLB` adjustments.
- Each claim's and each service's charge minus payment against their `CAS` adjustments.

The claim rule does not model interest (`AMT*I`): a claim whose payment includes interest does
not balance and is refused unless `allow_findings` is set.

## Null and empty text

In text columns, `null` is an element the segment does not have and `""` one it has but leaves
empty. A null cell in the middle of a segment is still written as an empty element, so it
reads back as `""`; only trailing nulls read back as `null`. The tables of a parse write back
to the same tables.
