# Reading large files

How to read a file one transaction or one claim at a time instead of all at once.

```python
import oxedi
from pathlib import Path

for batch in oxedi.stream(Path("big.835").read_bytes()):
    services = batch.tables["services"].to_polars()
```

Each batch holds the tables and diagnostics of one transaction. Pass `by="2100"` (or any
other loop id) to get one batch per claim instead. Parsing releases the GIL, so several files
can be parsed in parallel from threads. `stream` accepts `spec=` like `parse`.
