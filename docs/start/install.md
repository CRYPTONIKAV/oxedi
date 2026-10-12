# Install

How to install oxedi for Python, with its optional extras. To use it from SQL instead, see
[the DuckDB extension](../duckdb/index.md#install).

```bash
pip install oxedi               # no Python dependencies
pip install "oxedi[polars]"     # with Polars
pip install "oxedi[pandas]"     # with pandas and pyarrow
pip install "oxedi[pyx12]"      # with pyx12 validation
pip install "oxedi[edi-835-parser]"  # drop-in layer for edi-835-parser users
```

Requires Python 3.11 or later. Wheels are available for Linux (x86_64 and aarch64, plus musl
on x86_64), macOS (Intel and Apple silicon) and Windows (x86_64).
