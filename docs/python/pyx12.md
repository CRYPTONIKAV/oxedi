# Validating with pyx12

[pyx12](https://github.com/azoner/pyx12) checks a file against the implementation guide.
oxedi can run it for you and return its findings as ordinary diagnostics.

```bash
pip install "oxedi[pyx12]"
```

`parse` never calls pyx12. `validate` does, when you ask.

## Validate a file

This example breaks the segment count of the example file, so both oxedi and pyx12 have
something to report:

```python
from pathlib import Path

import oxedi
from oxedi.pyx12 import validate

data = Path("remittance.835").read_bytes().replace(b"SE*58*", b"SE*57*")
result = oxedi.parse(data)
findings = result.diagnostics + validate(data)    # bytes, a path or a binary file object

for d in sorted(findings, key=lambda d: d.level):
    print(d.origin, d.code, d)                    # "oxedi" or "pyx12"; pyx12's own code
    if d.segment is not None:
        start, end = result.document[d.segment].span  # the segment's bytes in your file
```

```text
oxedi None SNIP 1 · SE01 declares "57" but the count is 58 · segment #59, element 1 · at interchange#1/group#1/transaction#1 · datum "57"
pyx12 4 SNIP 1 · SE count of 57 for SE02=0001 is wrong. I count 58 (reported by pyx12, code 4) · segment #59, element 1 · at interchange#1/group#1/transaction#1 · datum "57"
pyx12 10 SNIP 2 · Data element "Implementation Convention Reference" (ST03) is marked as Not Used (reported by pyx12, code 10) · segment #2, element 3 · at interchange#1/group#1/transaction#1 · datum "005010X221A1"
```

pyx12 also reports `ST03`, which its map marks as not used. It does so on the unedited file too.

- `validate` returns `oxedi.Diagnostic`, the type `parse` returns. The two lists mix, sort by
  `level` and filter by `origin`.
- `validate` returns its findings in segment order.
- A file with no `ISA` raises `oxedi.ParseError`, as `parse` does.

## What a pyx12 finding holds

`validate` reports every error pyx12's engine records, read from the error tree pyx12 builds:

- Interchange, group and transaction errors. These are level 1.
- Segment and element errors, those of the envelope segments included. These are level 2.

A pyx12 finding has:

- `kind == "External"`.
- `origin == "pyx12"`.
- `code` set to pyx12's error code.
- `path` naming the loops open at its segment, as `parse` names them.

## Where a finding points

- A count or control number that does not match lands on the trailer segment, at the element
  holding it, with that value as `datum`.
- A duplicate or missing control structure lands on the header, at the control number when
  that is the offending value.
- When pyx12 gives no value, `datum` is the element's bytes, or the segment id for a finding
  with no element.

## When pyx12 cannot read a file

A file pyx12 cannot read, or an exception inside pyx12, comes back as one level 1 finding
instead of a traceback. It has no `code`, and its `rule` starts with
`could not finish validating`.

- A file pyx12 rejects points at its `ISA` segment, with the `ISA` bytes as `datum`.
- A failure after the last segment names the last segment pyx12 completed.
- A failure while reading names the segment pyx12 was processing.

??? note "Details: logging"
    pyx12 also logs what it finds, under the `pyx12` logger. On first use, `validate` attaches
    a `logging.NullHandler` to that logger when it has no handler. So the records are not
    printed by default, and they reach any handler you configure. This also applies to code
    that runs pyx12 directly in the same process. No level, propagation or `logging.disable`
    setting is changed.
