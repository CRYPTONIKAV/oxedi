# Install

oxedi needs Python 3.11 or later. Install it with pip:

```bash
pip install oxedi
```

The base package has no Python dependencies. Add an extra for the library you work with:

| Install | Adds | Use it to |
|---|---|---|
| `pip install "oxedi[polars]"` | Polars | get tables as Polars frames |
| `pip install "oxedi[pandas]"` | pandas and pyarrow | get tables as pandas frames |
| `pip install "oxedi[pyx12]"` | pyx12 | [validate with pyx12](../python/pyx12.md) |
| `pip install "oxedi[edi-835-parser]"` | a drop-in layer | [keep code written for edi-835-parser](../migrating-from-edi-835-parser.md) |

You can combine extras: `pip install "oxedi[polars,pyx12]"`.

## Platforms

Wheels are available for:

- Linux: x86_64 and aarch64, plus musl on x86_64.
- macOS: Intel and Apple silicon.
- Windows: x86_64.

## DuckDB

To use oxedi from SQL, install the DuckDB extension instead. See
[DuckDB: install](../duckdb/index.md#install).

Next: [Get started](index.md).
