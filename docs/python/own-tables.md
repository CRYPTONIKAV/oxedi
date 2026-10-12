# Writing from your own tables

Your tables do not have to come from a parse. This page shows how to shape them so
`oxedi.write` or `COPY` accepts them. Read [Writing 835 files](writing.md) first for the
envelope.

## What the writer accepts

`oxedi.write` and `COPY` take the spec's table and column names and nothing else.

Tolerated:

- A missing table. It has no rows.
- A missing column. It is null.
- A column of a narrower or compatible type: an integer for a big integer, or a decimal of
  another scale that holds the value.
- In SQL, `INTEGER` for a `BIGINT` column, and `ENUM` or `BLOB` for a text column.

Refused, naming the table, row and column:

- A table or column that is not in the spec.
- Floats for money.
- A value that would change in the conversion.
- A file whose totals do not balance or that lacks a required element.

## Number your rows

The spec's tables key their rows by position. If your tables have their own keys, translate
them:

1. Number the rows of each table from 0 in a `row` column.
2. Bring the parents' `row` in with a join, into `payment`, `claim` and `service`.
3. Drop your own key columns.

Order matters:

- Rows of one parent must be together and in their parents' order.
- Sort each child table by the parent's `row` first, then by its own key. Keep the table's
  order for ties.
- The adjustments of a claim (no `service`) come before those of its services.

!!! note "Provider adjustments and `PLB`"
    Keep the `segment` column of `provider_adjustments` when you have one. Adjustments with the
    same `segment` are written in one `PLB`. Without it, all the adjustments of a payment share
    one `PLB`.

## See the columns a table needs

Parse any file and look at its tables. [Tables and columns](../reference/tables.md) lists them
all.

=== "Python"

    ```python
    import oxedi

    result = oxedi.parse_file("remittance.835")
    print(result.tables["claims"].to_polars().schema)
    ```

=== "SQL"

    ```sql
    DESCRIBE SELECT * FROM read_835('remittance.835', table_name := 'claims');
    ```

`result.tables["claims"].columns` gives the names alone. A schema that does not need a file is
tracked in
[#138](https://github.com/javillegasna/oxedi/issues/138).

## Translate your keys

The examples take five tables of your own:

- `my_payments`, keyed by `pay_key`.
- `my_claims`, keyed by `claim_key`, with its `pay_key`.
- `my_services`, keyed by `service_key`, with its `claim_key`.
- `my_adjustments`, with its `claim_key`, and `service_key` when the adjustment is on a service.
- `my_provider_adjustments`, with its `pay_key`.

Each also has the spec's other columns.

=== "Polars"

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

=== "SQL"

    Give each table a `row` with `row_number()`. Put the parents' `row` in its parent columns
    with a join, and drop your helper columns with `EXCLUDE`.

    - Number each child table ordered by the parent's `row`, then by its own key, then by a
      stable tie-breaker: a sequence column, or `rowid` of a table.
    - The adjustments of a claim, which have no `service`, come first (`NULLS FIRST`).
    - Your keys need not sort in parent order across parents.

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
