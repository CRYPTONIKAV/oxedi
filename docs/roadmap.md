# Roadmap

Where oxedi is and where it is going. Nothing here is a promise or a date: plans change with
what people need. If something matters to you, say so in an
[issue](https://github.com/javillegasna/oxedi/issues). It is the best way to move it up.

## Available now

- **Read an 835** from Python into payments, claims, service lines, adjustments and provider
  adjustments, as tables for Polars, pandas, pyarrow or DuckDB. Every byte of the file is kept,
  and problems come back as diagnostics instead of errors. See [Reading](python/reading.md).
- **Write an 835** from those same tables, with control numbers, counts and balancing checked
  before anything is written. See [Writing](python/writing.md).
- **Use SQL** with the DuckDB extension: read files, folders, globs or cloud storage with
  `read_835`, and write an 835 with `COPY`. See [DuckDB](duckdb/index.md).
- **Validate with pyx12** and get its findings in the same shape as oxedi's own. See
  [Validating with pyx12](python/pyx12.md).
- **Adapt to your payers** by changing the spec, which is plain data. See [The spec](customize/spec.md).
- **Try it in your browser.** Drop an 835 on the [viewer](https://javillegas.dev/oxedi/viewer/)
  and see its payments, tables, problems and loops. The file is read in the browser and never
  uploaded, so it is safe to use with real remittances. See [Try it in your browser](start/viewer.md).
- **Hide patient data.** Replace patient names and ids with keyed tokens and reduce dates, for
  sharing data with analysts, vendors or tests. It works in Python (`parse`, `validate`) and in
  the browser viewer, and comes to DuckDB with the next extension version. See
  [Sharing data without patient information](python/without-patient-data.md).
- **Move from edi-835-parser** without rewriting your code. See
  [Migrating from edi-835-parser](migrating-from-edi-835-parser.md).

## Next

- **Ask an AI assistant about your remittances.** A local tool server that lets an assistant
  answer questions such as totals by payer, denied claims or a claim's adjustments, over a
  file, a folder or cloud storage. Patient fields would be masked by default.

## Being considered

- **More X12 transactions**, starting with the 837 claim. The same engine, a different spec,
  and a way to match claims with their payments.
- **More recipes**, such as Spark and Ray for very large batches.
- **Other languages** beyond Python and DuckDB, such as a JavaScript package or a C library,
  when someone needs them.
- **Very large files** read in pieces instead of all at once.
