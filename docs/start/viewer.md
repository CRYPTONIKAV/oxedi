# Try it in your browser

You can read an 835 file without installing anything. Open the
[oxedi viewer](https://javillegas.dev/oxedi/viewer/), drop a file on it and look at the payment,
the tables and the problems oxedi found.

## Open a file

1. Open <https://javillegas.dev/oxedi/viewer/>.
2. Drop an 835 file on the page, or click it to choose one. The file can have any extension.
3. Press **Try the sample** if you only want to look around. The sample is a small synthetic file.

If the file is not an 835, the page shows the parser's message and keeps working, so you can
drop the next file.

## What you see

The viewer runs the same parser as the Python package, compiled to WebAssembly. A file opens
in five views:

| View | What it shows |
|---|---|
| Summary | The payer, the payee, the payment method, the date, the total and the counts. |
| Tables | Payments, claims, service lines and adjustments. You can filter the rows and download them as CSV. |
| Diagnostics | Each problem oxedi found, with the rule, the position and the value. |
| Text | The original file, one segment per line. |
| Loops | The loop structure, which you can fold. |

A diagnostic links to its segment in the Text view, with the element it names highlighted. A
`segment` cell in a table does the same. Every byte of the file is shown, including line breaks
after a terminator and a last segment that has none.

Summary also shows which version of the spec the page used, 5010 or 4010, chosen from the file.

## Your file stays on your machine

The page has no server behind it. It reads the file in your browser and never sends it
anywhere. It also asks the browser to forbid network connections, so even a bug could not send
the file out.

??? note "Check it yourself"
    1. Open the browser's developer tools and go to the **Network** tab.
    2. Drop a file on the page.
    3. No request appears after the page has loaded.

    The page carries a Content-Security-Policy with `connect-src 'none'`, which you can read in
    the page's source. It uses no analytics and no third-party fonts or scripts.

## What it does not do

- It only reads. It does not edit or write an 835.
- It opens one file at a time.
- It does not validate with pyx12, which is a Python library. Use the
  [Python package](../python/pyx12.md) for that.

!!! note "Large files"
    A file over 20 MB makes the page ask before it parses. The largest sample in the
    repository, about 600 KB with 30 000 segments, opens in well under a second on a laptop.

## Next

- [Get started](index.md): parse the same file in Python.
- [The 835 in five minutes](the-835.md): what the segments and loops mean.
