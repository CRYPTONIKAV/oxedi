import shutil
from pathlib import Path

import pytest

from blocks import ROOT, pages

FIXTURES = ROOT / "crates" / "oxedi_core" / "tests" / "fixtures"

# Conventional file names the pages use, and the fixture each one is a copy of.
FILES = {
    "remittance.835": "balanced_5010_sample.txt",
    "remittance-4010.835": "balanced_4010_sample.txt",
    "big.835": "balanced_5010_sample.txt",
    "a.835": "balanced_5010_sample.txt",
    "b.835": "balanced_4010_sample.txt",
    "remits/a.835": "balanced_5010_sample.txt",
    "remits/b.835": "balanced_4010_sample.txt",
    "remits/c.835": "emedny_sample.txt",
    "remittances/a.835": "balanced_5010_sample.txt",
    "remittances/b.835": "balanced_4010_sample.txt",
}


@pytest.fixture
def workdir(tmp_path, monkeypatch):
    """A fresh directory holding the conventional files, as the working directory."""
    for name, source in FILES.items():
        target = tmp_path / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(FIXTURES / source, target)
    monkeypatch.chdir(tmp_path)
    return tmp_path


def pytest_terminal_summary(terminalreporter):
    write = terminalreporter.write_line
    terminalreporter.section("documentation examples")
    total_run = total_skipped = 0
    for page in pages():
        run = {lang: page.count(lang) for lang in ("python", "sql")}
        skipped = len(page.skipped)
        total_run += sum(run.values())
        total_skipped += skipped
        write(f"{page.name}: python {run['python']}, sql {run['sql']}, skipped {skipped}")
    write(f"blocks run: {total_run}, skipped: {total_skipped}")
    for page in pages():
        for lang, reason, first in page.skipped:
            write(f"  skipped {page.name} [{lang}] {first!r}: {reason}")
