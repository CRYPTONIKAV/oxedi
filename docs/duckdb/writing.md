# Writing with COPY

How to write an 835 from SQL with `COPY ... (FORMAT edi835)`, its options and strict mode.

Writing with `COPY` needs extension 0.2.0 or later; until it reaches the community repository,
build the extension.

`COPY ... TO ... (FORMAT edi835, ...)` writes an 835 with the same writer as the Python
package's `oxedi.write`; the file is byte for byte what `oxedi.write` gives for the same
tables and envelope. The query returns **one `STRUCT` column** whose fields are named after the
spec's tables, each a list of structs (one struct per row, the table's columns as fields).
This example reads the tables of a file into temporary tables, then writes them back:

```sql
CREATE TEMP TABLE payments AS FROM read_835('remittance.835', table_name := 'payments');
CREATE TEMP TABLE claims AS FROM read_835('remittance.835', table_name := 'claims');
CREATE TEMP TABLE services AS FROM read_835('remittance.835', table_name := 'services');
CREATE TEMP TABLE adjustments AS FROM read_835('remittance.835', table_name := 'adjustments');
CREATE TEMP TABLE provider_adjustments AS
  FROM read_835('remittance.835', table_name := 'provider_adjustments');
```

```sql
COPY (
  SELECT {
    'payments':             (SELECT list(t ORDER BY t."row") FROM payments t),
    'claims':               (SELECT list(t ORDER BY t."row") FROM claims t),
    'services':             (SELECT list(t ORDER BY t."row") FROM services t),
    'adjustments':          (SELECT list(t ORDER BY t."row") FROM adjustments t),
    'provider_adjustments': (SELECT list(t ORDER BY t."row") FROM provider_adjustments t)
  }
) TO 'out.835' (FORMAT edi835, sender_id 'ACMEPAYER', receiver_id 'SUNRISECLINIC',
                date DATE '2024-01-10', time TIME '09:00');
```

`payments`, `claims` and the rest can be any tables or views with those names, built in SQL
or read from a file as above and changed before writing. Temporary tables, views and the open
transaction are visible. The lists of several rows of the query concatenate into one
interchange. A table left out has no rows and a column left out is null.

To write from tables with your own keys, see
[Writing from your tables](../python/own-tables.md#with-sql-duckdb).

## Options

| Option | Default | Meaning |
|---|---|---|
| `sender_id`, `receiver_id` | required | interchange sender and receiver ids |
| `date`, `time` | required | the interchange date (`DATE`) and time (`TIME` or `TIME_NS`) |
| `sender_qualifier`, `receiver_qualifier` | `'ZZ'` | ISA qualifiers |
| `usage_indicator` | `'P'` | `'P'` production or `'T'` test |
| `control_number` | `1` | first interchange control number |
| `application_sender`, `application_receiver` | the ids | group header sender and receiver codes |
| `delimiters` | `*` `:` `~` `^` | a struct such as `{'element': '\|', 'repetition': '^'}` with any of `element`, `component`, `segment`, `repetition`, `release`; fields left out take the defaults of `oxedi.Delimiters()` (`*`, `:`, `~`, and no repetition or release), so on 5010 give `repetition` |
| `line_break` | `false` | a line break after each segment |
| `version` | `'5010'` | the built-in spec, `'5010'` or `'4010'` |

The options are those of `oxedi.Envelope`, with the same defaults.

## Strict

The written file is read back with the spec, and every diagnostic of that read is a finding, as
is anything the writer cannot place (a missing required element, a code outside its list, a row
whose parent does not exist or is out of order, money that does not balance). Any finding fails
the `COPY` with the listing `oxedi.WriteError` shows, each finding naming the table, row and
column, and nothing is written. There is no `allow_findings` here yet.

## From your own tables

The writer asks for the spec's names and nothing else, so tables built elsewhere work when they
follow them (see [Writing](../python/own-tables.md)):

- **Tolerated:** a missing table (no rows), a missing column (null), a column of a narrower or
  compatible type (`INTEGER` for `BIGINT`, a `DECIMAL` of another scale that holds the value,
  `ENUM` or `BLOB` for text).
- **Refused, naming the table, row and column:** a table or column that is not in the spec, a float
  for money, a value that would change in the conversion (a decimal that does not fit, a number
  outside the column's range), and a file that does not balance or is missing a required
  element.
