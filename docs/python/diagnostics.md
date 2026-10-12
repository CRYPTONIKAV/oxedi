# Diagnostics

What the parser reports about questionable data, and how to read each finding.

```python
import oxedi

result = oxedi.parse_file("remittance.835")
```

A file that is valid EDI but has questionable data still parses. Each issue is returned in
`result.diagnostics` (and in each batch of `stream`) instead of being raised:

```python
for diagnostic in result.diagnostics:
    print(diagnostic)
```

A diagnostic names the rule that failed, where it failed and the offending value. Its fields:

- `level`: the SNIP level of the rule, 1, 2 or 3; sort by it to read the lowest first.
- `kind`: the name of the rule that failed, for example `RequiredElementMissing`.
- `rule`: the rule as a sentence with its values.
- `segment`: the index of the segment at fault, or `None` at the end of the file.
- `element` and `component`: the 1-based position inside the segment, when the finding has one.
- `path`: the loops open at that segment, outermost first, joined by `/`; empty at the root.
- `datum`: the offending value as it appears in the file (bytes).
- `origin` and `code`: who reported it (`"oxedi"`, or `"pyx12"` with pyx12's own code; see
  [Validating with pyx12](pyx12.md)).

To find the bytes of the segment a diagnostic points at, use the document (see [Reading](reading.md#the-document-and-writing-the-file-back)):
`result.document[d.segment].span` is the segment's byte range in your file.
