"""Runs every `python` and `sql` block of docs/**/*.md and README.md.

The blocks of a page run in order, in one namespace (Python) or one DuckDB connection
(SQL), inside a directory holding the conventional example files. Build the SQL
extension first with `make release`, or point OXEDI_EXTENSION at a built file.
"""

import os
import re

import pytest

from blocks import ROOT, pages

PAGES = pages()
EXTENSION = ROOT / "build" / "release" / "extension" / "oxedi" / "oxedi.duckdb_extension"


def with_blocks(lang):
    return [pytest.param(page, id=page.name) for page in PAGES if page.count(lang)]


def test_markers_are_well_formed():
    problems = [problem for page in PAGES for problem in page.problems]
    assert not problems, "\n".join(problems)


@pytest.mark.parametrize("page", PAGES, ids=lambda page: page.name)
def test_every_fence_is_run_or_skipped(page):
    run = sum(page.count(lang) for lang in ("python", "sql"))
    assert run + len(page.skipped) == page.fenced, (
        f"{page.name} has {page.fenced} python and sql fences but the runner found "
        f"{run} to run and {len(page.skipped)} skipped"
    )


@pytest.mark.parametrize("page", with_blocks("python"))
def test_python_blocks(page, workdir):
    namespace = {"__name__": "__main__"}
    for block in page.run["python"]:
        label = f"{page.name}:{block.line}, python block"
        try:
            exec(compile(block.code, label, "exec"), namespace)
        except Exception as error:
            error.add_note(f"{label}:\n{block.code}")
            raise


@pytest.mark.parametrize("page", with_blocks("sql"))
def test_sql_blocks(page, workdir):
    import duckdb

    extension = os.environ.get("OXEDI_EXTENSION", str(EXTENSION))
    if not os.path.exists(extension):
        pytest.fail(f"no DuckDB extension at {extension}; run `make release` first")
    connection = duckdb.connect(config={"allow_unsigned_extensions": "true"})
    connection.execute(f"LOAD '{extension}'")
    for block in page.run["sql"]:
        label = f"{page.name}:{block.line}, sql block"
        try:
            cursor = connection.execute(block.code)
            if cursor.description:
                cursor.fetchall()
        except Exception as error:
            error.add_note(f"{label}:\n{block.code}")
            raise


def test_api_reference_covers_the_public_api():
    import oxedi
    import oxedi.edi_835_parser
    import oxedi.pyx12

    text = "\n".join(path.read_text() for path in sorted((ROOT / "docs" / "reference" / "api").glob("*.md")))
    documented = set(re.findall(r"^::: (\S+)", text, re.MULTILINE))
    expected = {f"oxedi.{name}" for name in oxedi.__all__ if name != "__version__"}
    expected |= {f"oxedi.pyx12.{name}" for name in oxedi.pyx12.__all__}
    expected |= {"oxedi.edi_835_parser"}
    assert not expected - documented, f"not in the API reference: {sorted(expected - documented)}"
    # The edi_835_parser page has no member list: it follows the module's __all__.
    page = (ROOT / "docs" / "reference" / "api" / "edi_835_parser.md").read_text()
    assert "members:" not in page
    assert oxedi.edi_835_parser.__all__
