# Writing from your own tables

How to number your tables the way the writer expects, with Polars or with SQL in DuckDB.
Start with [Writing 835 files](writing.md) for the envelope.

`oxedi.write` takes the spec's table and column names and nothing else.

- **Tolerated:** a missing table (no rows), a missing column (null) and a column of a narrower
  or compatible type (an integer for a big integer, a decimal of another scale that holds the
  value).
- **Refused, naming the table, row and column:** a table or column that is not in the spec,
  floats for money, a value that would change in the conversion, and a file whose totals do
  not balance or that lacks a required element.

The spec's tables key their rows by position: `row` counts the rows of a table from 0, and
`payment`, `claim` and `service` hold the `row` of the parent. If your tables have their own
keys, number the rows, bring the parents' `row` in with a join and drop your helper columns.
Rows of one parent must be together and in their parents' order, so sort each child table by
the parent's `row` first, then by its own key (keep the table's order for ties). The
adjustments of a claim (no `service`) come before those of its services. Keep the `segment`
column of `provider_adjustments` when you have one: adjustments with the same `segment` are
written in one `PLB`, and without it all the adjustments of a payment share one `PLB`.

To see a table's columns, use `result.tables["claims"].to_polars().schema` (or `.columns`) on
any parsed file, or in the extension
`DESCRIBE SELECT * FROM read_835('remittance.835', table_name := 'claims')`. A schema that
does not need a file is tracked in [#138](https://github.com/javillegasna/oxedi/issues/138).

## With Polars

Use `with_row_index("row")` to number the rows.

<!-- docs-test: skip needs the reader's own tables (my_payments and the others), which the example files do not provide -->
```python
import polars as pl

# my_payments (pay_key), my_claims (claim_key, pay_key), my_services (service_key,
# claim_key), my_adjustments (claim_key, service_key or null), my_provider_adjustments
# (pay_key): your tables, each with the spec's other columns
payments = my_payments.sort("pay_key").with_row_index("row")
claims = (
    my_claims.join(payments.select("row", "pay_key"), on="pay_key", maintain_order="left")
    .rename({"row": "payment"})
    .sort("payment", "claim_key", maintain_order=True)
    .with_row_index("row")
)
claim_rows = claims.select(claim="row", payment="payment", claim_key="claim_key")
services = (
    my_services.join(claim_rows, on="claim_key", maintain_order="left")
    .sort("claim", "service_key", maintain_order=True)
    .with_row_index("row")
)
adjustments = (
    my_adjustments.join(claim_rows, on="claim_key", maintain_order="left")
    .join(
        services.select(service="row", service_key="service_key"),
        on="service_key", how="left", maintain_order="left",
    )
    .sort("claim", "service", nulls_last=False, maintain_order=True)
    .with_row_index("row")
)
provider_adjustments = (
    my_provider_adjustments.join(
        payments.select(payment="row", pay_key="pay_key"), on="pay_key", maintain_order="left"
    )
    .sort("payment", maintain_order=True)
    .with_row_index("row")
)
frames = {
    "payments": payments.drop("pay_key"),
    "claims": claims.drop("pay_key", "claim_key"),
    "services": services.drop("service_key", "claim_key"),
    "adjustments": adjustments.drop("claim_key", "service_key"),
    "provider_adjustments": provider_adjustments.drop("pay_key"),
}
data = oxedi.write(frames, envelope)
```

## With SQL (DuckDB)

Give each table a `row` with `row_number()`, put the parents' `row` in its parent columns with a
join, and drop your helper columns with `EXCLUDE`. Number each child table ordered by the
parent's `row` first, then by its own key, then by a stable tie-breaker (a sequence column, or
`rowid` of a table). The adjustments of a claim, which have no `service`, come before those of
its services (`NULLS FIRST`). Your keys need not sort in parent order across parents.

This example takes `my_payments` (key `pay_key`), `my_claims` (`claim_key`, `pay_key`),
`my_services` (`service_key`, `claim_key`), `my_adjustments` (`claim_key`, and `service_key`
when the adjustment is on a service) and `my_provider_adjustments` (`pay_key`), each with the
spec's other columns:

<!-- docs-test: skip needs the reader's own tables (my_payments and the others), which the example files do not provide -->
```sql
COPY (
  WITH payments AS (
         SELECT row_number() OVER (ORDER BY pay_key) - 1 AS "row", * FROM my_payments),
       claims AS (
         SELECT row_number() OVER (ORDER BY p."row", c.claim_key) - 1 AS "row", p."row" AS payment, c.*
         FROM my_claims c JOIN payments p USING (pay_key)),
       services AS (
         SELECT row_number() OVER (ORDER BY c."row", s.service_key) - 1 AS "row",
                c.payment, c."row" AS claim, s.*
         FROM my_services s JOIN claims c USING (claim_key)),
       adjustments AS (
         SELECT row_number() OVER (ORDER BY c."row", s."row" NULLS FIRST, a.rowid) - 1 AS "row",
                c.payment, c."row" AS claim, s."row" AS service, a.*
         FROM my_adjustments a
         JOIN claims c USING (claim_key)
         LEFT JOIN services s ON s.service_key = a.service_key),
       provider_adjustments AS (
         SELECT row_number() OVER (ORDER BY p."row", a.rowid) - 1 AS "row", p."row" AS payment, a.*
         FROM my_provider_adjustments a JOIN payments p USING (pay_key))
  SELECT {
    'payments': (SELECT list(t ORDER BY t."row") FROM (SELECT * EXCLUDE (pay_key) FROM payments) t),
    'claims': (SELECT list(t ORDER BY t."row")
               FROM (SELECT * EXCLUDE (pay_key, claim_key) FROM claims) t),
    'services': (SELECT list(t ORDER BY t."row")
                 FROM (SELECT * EXCLUDE (service_key, claim_key) FROM services) t),
    'adjustments': (SELECT list(t ORDER BY t."row")
                    FROM (SELECT * EXCLUDE (claim_key, service_key) FROM adjustments) t),
    'provider_adjustments': (SELECT list(t ORDER BY t."row")
                             FROM (SELECT * EXCLUDE (pay_key) FROM provider_adjustments) t)
  }
) TO 'out.835' (FORMAT edi835, sender_id 'ACMEPAYER', receiver_id 'SUNRISECLINIC',
                date DATE '2024-01-10', time TIME '09:00');
```
