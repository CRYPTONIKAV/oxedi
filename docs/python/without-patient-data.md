# Sharing data without patient information

An 835 names the patient: name, member id, claim number and service dates. Pass a mask to
oxedi and it replaces those values in the tables, and in the diagnostics that quote them, with
tokens and coarser dates. You can then share the tables, attach them to a ticket or build
synthetic test files from them.

```python
import oxedi

mask = oxedi.Mask("a-secret-of-at-least-16-bytes", dates="year")
result = oxedi.parse_file("remittance.835", mask=mask)
```

## What is masked

The spec flags the columns that hold patient data, so the list below is the built-in spec's. A
patched spec can flag others (see [Customizing the spec](../customize/spec.md#flag-patient-data)).

| Table | Column | Kind |
|---|---|---|
| `claims` | `patient_last_name`, `patient_first_name` | name |
| `claims` | `patient_id`, `claim_id`, `payer_control_number`, `group_number` | id |
| `claims` | `statement_from`, `statement_to` | date |
| `services` | `line_item_control_number` | id |
| `services` | `service_date` | date |
| `provider_adjustments` | `reference_id` | id |

A name or an id becomes 16 lowercase hexadecimal characters: the first 8 bytes of an HMAC-SHA256
of the message `kind`, a zero byte, then the value (`kind` is `name` or `id`), keyed with your key,
so a token can be reproduced from the key. Two consequences follow:

- The same value and key always give the same token, so a masked `patient_id` still joins the
  same patient across claims and files.
- Without the key the original cannot be recovered from a token.

An empty value stays empty, and a missing value stays null.

The diagnostics are masked too. The value a finding quotes is masked when it comes from `DTM02`,
`REF02`, `NM1` elements 3, 4 and 9, `CLP01` or `CLP07`. This covers every segment of those types,
so it also hides the value of a finding about the payer, the payee or a provider, such as one
about a `DTM*405` date or a payer `REF02`. Amounts a finding quotes stay as they are.
A `LengthOutOfRange` finding still states the length of the original value. With
`dates="shift"`, a date inside a diagnostic is reduced to its year.

## Choose what happens to dates

| `dates` | Result |
|---|---|
| `"year"` (default) | January 1 of the same year. |
| `"shift"` | A claim and its service lines move by the same number of days, between -182 and +182, derived from the key and the claim's patient id. Intervals inside a claim are kept. |
| `"keep"` | The date is unchanged. |

A claim with no patient id cannot be shifted: its dates and those of its service lines fall back
to `"year"`.

```python
import polars as pl

key = "a-secret-of-at-least-16-bytes"
for dates in ("year", "shift"):
    result = oxedi.parse_file("remittance.835", mask=oxedi.Mask(key, dates=dates))
    claims = result.tables.to_polars()["claims"]
    print(dates, claims.select(pl.col("patient_last_name").cast(pl.String), "statement_from").row(0))
```

```text
year ('2fc81fb06ff98c31', datetime.date(2024, 1, 1))
shift ('2fc81fb06ff98c31', datetime.date(2023, 9, 2))
```

## The key

The key is bytes or text of at least 16 bytes. A shorter one raises `ValueError`
("the mask key must be at least 16 bytes long; got 5"). There is no default key, so a mask
never exists by accident.

Keep the key out of the files you share, and reuse it when you need the same token for the same
patient in another run.

`oxedi.parse`, `oxedi.parse_file` and `oxedi.stream` all take `mask=` and mask the same way.

## Use it in SQL

`read_835` takes `mask_key` and `mask_dates` (`'year'`, `'shift'` or `'keep'`). A NULL key is
refused, and `mask_dates` without `mask_key` is an error.

```sql
SET VARIABLE mask_key = 'a-secret-of-at-least-16-bytes';

SELECT patient_last_name, statement_from
FROM read_835('remittance.835', mask_key := getvariable('mask_key'), mask_dates := 'year')
LIMIT 2;
```

```text
┌───────────────────┬────────────────┐
│ patient_last_name │ statement_from │
│      varchar      │      date      │
├───────────────────┼────────────────┤
│ 2fc81fb06ff98c31  │ 2024-01-01     │
│ 924149a1fc712857  │ 2024-01-01     │
└───────────────────┴────────────────┘
```

!!! warning "A literal key stays in the query history"
    A key written in the SQL text can appear in `EXPLAIN ANALYZE` output, in the `LINE 1:`
    excerpt of an error message and in your client's query history. `SET VARIABLE` with a
    literal, as above, keeps it in the history too. Pass the key as a prepared-statement
    parameter instead, so it never enters the SQL text:

    <!-- docs-test: skip needs the DuckDB extension loaded in the connection -->
    ```python
    import duckdb

    con = duckdb.connect()                    # with the oxedi extension loaded
    rows = con.execute(
        "SELECT patient_last_name FROM read_835(?, mask_key := ?, mask_dates := ?)",
        ["remittance.835", key, "shift"],
    ).fetchall()
    ```

## What is not masked

- **The raw file.** `result.document` and every `Segment` keep the original bytes, because oxedi
  never changes them. Do not share `result.document.write()`.
- **Payer, payee and provider data.** Names, addresses and identification codes of the payer, the
  payee and the rendering provider are not patient data and stay as they are.
- **Findings from `oxedi.pyx12.validate`.** They quote pyx12's own text and are not masked.

With the default `dates="year"`, this follows the HIPAA Safe Harbor identifiers that appear in the
tables: names, member and claim numbers, and dates reduced to the year. `"shift"` and `"keep"`
do not remove dates the way Safe Harbor requires. It is not a legal de-identification
determination. Free text, rare combinations of values and the data you add yourself are still
yours to review.

## Make synthetic test files

Masked tables are valid input for the writer, so you can turn a real file into a shareable one:

```python
import datetime
import oxedi

result = oxedi.parse_file("remittance.835", mask=oxedi.Mask(key))
envelope = oxedi.Envelope(
    sender_id="ACMEPAYER",
    receiver_id="SUNRISECLINIC",
    date=datetime.date(2024, 1, 10),
    time=datetime.time(9, 0),
    usage_indicator="T",
)
data = oxedi.write(result.tables.to_polars(), envelope, spec=result.spec)
print(oxedi.parse(data).diagnostics)
```

```text
[]
```

## In the browser

The [viewer](../start/viewer.md) has a **Hide patient data** switch that applies the same mask
to the tables and their CSV export.
