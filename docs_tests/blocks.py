"""Finds the `python` and `sql` code blocks of the documentation pages.

A block is skipped by putting this comment on the line right before its opening fence:

    <!-- docs-test: skip <reason> -->

The reason is required. Every skipped block is listed in the test run's summary.
"""

import re
from dataclasses import dataclass, field
from pathlib import Path

from mktestdocs import grab_code_blocks

ROOT = Path(__file__).resolve().parent.parent
LANGS = ("python", "sql")

MARKER = re.compile(r"^<!--\s*docs-test:\s*skip\b(?P<reason>.*?)\s*-->\s*$")
FENCE = re.compile(r"^```(?P<lang>python|sql)\s*$")


@dataclass
class Page:
    path: Path
    run: dict = field(default_factory=dict)  # language -> blocks to execute, in page order
    skipped: list = field(default_factory=list)  # (language, reason, first line of the block)
    problems: list = field(default_factory=list)  # malformed markers
    fenced: int = 0  # python and sql fences counted straight from the text

    @property
    def name(self) -> str:
        return self.path.relative_to(ROOT).as_posix()

    def count(self, lang: str) -> int:
        return len(self.run.get(lang, []))


def pages() -> list:
    paths = sorted((ROOT / "docs").rglob("*.md")) + [ROOT / "README.md"]
    return [read(path) for path in paths]


def read(path: Path) -> Page:
    page = Page(path)
    lines = path.read_text().split("\n")
    page.fenced = sum(1 for line in lines if FENCE.match(line))
    for number, line in enumerate(lines, start=1):
        marker = MARKER.match(line)
        if not marker:
            continue
        following = lines[number] if number < len(lines) else ""
        fence = FENCE.match(following)
        if not marker.group("reason").strip():
            page.problems.append(f"{page.name}:{number}: a skip marker needs a reason")
        if not fence:
            page.problems.append(f"{page.name}:{number}: a skip marker must sit right before a python or sql fence")
            continue
        body = [l for l in lines[number + 1 :]]
        first = next((l.strip() for l in body if l.strip()), "")
        page.skipped.append((fence.group("lang"), marker.group("reason").strip(), first))
        # Rename the fence so the extractor passes over this block.
        lines[number] = f"```skipped-{fence.group('lang')}"
    text = "\n".join(lines)
    for lang in LANGS:
        found = grab_code_blocks(text, lang=lang)
        if found:
            page.run[lang] = found
    return page
