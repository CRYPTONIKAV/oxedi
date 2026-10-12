# Roadmap

Where oxedi is and where it is going. Nothing here is a promise or a date: plans change with
what people need. If something matters to you, say so in an
[issue](https://github.com/javillegasna/oxedi/issues). It is the best way to move it up.

## Available now

- **Read an 835** from Python into payments, claims, service lines, adjustments and provider
  adjustments, as tables for Polars, pandas, pyarrow or DuckDB. Every byte of the file is kept,
  and problems come back as diagnostics instead of errors. See [Reading](reading.md).
- **Write an 835** from those same tables, with control numbers, counts and balancing checked
  before anything is written. See [Writing](writing.md).
- **Use SQL** with the DuckDB extension: read files, folders, globs or cloud storage with
  `read_835`, and write an 835 with `COPY`. See [DuckDB](duckdb.md).
- **Validate with pyx12** and get its findings in the same shape as oxedi's own. See
  [Validating with pyx12](pyx12.md).
- **Adapt to your payers** by changing the spec, which is plain data. See [The spec](spec.md).
- **Move from edi-835-parser** without rewriting your code. See
  [Migrating from edi-835-parser](migrating-from-edi-835-parser.md).

## Next

- **A viewer in the browser.** Drop an 835 on a web page and see its payments, tables,
  problems and loops. The file is read in the browser and never uploaded, so it is safe to use
  with real remittances. No install needed to try oxedi.

## Being considered

- **Ask an AI assistant about your remittances.** A local tool server that lets an assistant
  answer questions such as totals by payer, denied claims or a claim's adjustments, over a
  file, a folder or cloud storage. Patient fields would be masked by default.
- **A mode without patient data.** Tables where patient names and ids are replaced by keyed
  hashes, for sharing data with analysts, vendors or tests.
- **Interest and prompt-pay discounts in claim balancing**, so claims that carry interest are
  checked correctly ([#129](https://github.com/javillegasna/oxedi/issues/129)).
- **More X12 transactions**, starting with the 837 claim. The same engine, a different spec,
  and a way to match claims with their payments.
- **More recipes**, such as Spark and Ray for very large batches.
- **Other languages** beyond Python and DuckDB, such as a JavaScript package or a C library,
  when someone needs them.
- **Very large files** read in pieces instead of all at once.
