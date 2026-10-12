"""Finds the `python` and `sql` code blocks of the documentation pages.

A block is skipped by putting this comment on the line right before its opening fence:

    <!-- docs-test: skip <reason> -->

The reason is required. Every skipped block is listed in the test run's summary.

Fences may use backticks or tildes, be indented (admonitions, tabs) and carry attributes
after the language. `py` and `python3` count as `python`. A `pycon` fence is a console
session, which cannot run, so it is reported as a problem instead of being ignored.
"""

import re
import textwrap
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LANGS = ("python", "sql")
ALIASES = {"python": "python", "py": "python", "python3": "python", "sql": "sql"}

MARKER = re.compile(r"^\s*<!--\s*docs-test:\s*skip\b(?P<reason>.*?)\s*-->\s*$")
OPENING = re.compile(r"^(?P<indent>\s*)(?P<fence>`{3,}|~{3,})\s*(?P<info>.*)$")
# What counts a fence of interest without going through the extractor above.
LOOKS_RUNNABLE = re.compile(r"^\s*(?:`{3,}|~{3,})\s*\{?\.?(?:python3?|py|pycon|sql)\b", re.IGNORECASE)


@dataclass
class Block:
    code: str
    line: int  # 1-based line of the opening fence


@dataclass
class Page:
    path: Path
    name: str
    run: dict = field(default_factory=dict)  # language -> blocks to execute, in page order
    skipped: list = field(default_factory=list)  # (language, reason, first line of the block)
    problems: list = field(default_factory=list)  # malformed markers and unsupported fences
    fenced: int = 0  # python and sql fences counted straight from the text

    def count(self, lang: str) -> int:
        return len(self.run.get(lang, []))


def pages() -> list:
    paths = sorted((ROOT / "docs").rglob("*.md")) + [ROOT / "README.md"]
    return [parse(path.read_text(), path.relative_to(ROOT).as_posix(), path) for path in paths]


def language(info: str) -> str:
    """The first word of a fence's info string, without a leading `{` or `.`."""
    match = re.match(r"[{.]*(\w+)", info.strip())
    return match.group(1).lower() if match else ""


def parse(text: str, name: str, path: Path = Path("")) -> Page:
    page = Page(path, name)
    lines = text.split("\n")
    page.fenced = sum(1 for line in lines if LOOKS_RUNNABLE.match(line))
    skip = None  # (line, reason) of a marker waiting for its fence
    open_fence = None  # (char, length, indent, lang, line, body)
    for number, line in enumerate(lines, start=1):
        if open_fence:
            char, length, indent, lang, start, body = open_fence
            stripped = line.strip()
            if stripped and set(stripped) == {char} and len(stripped) >= length:
                open_fence = None
                if lang:
                    code = textwrap.dedent("\n".join(body)) + "\n"
                    if skip:
                        first = next((l.strip() for l in body if l.strip()), "")
                        page.skipped.append((lang, skip[1], first))
                    else:
                        page.run.setdefault(lang, []).append(Block(code, start))
                    skip = None
            else:
                body.append(line)
            continue
        opening = OPENING.match(line)
        if opening:
            word = language(opening.group("info"))
            lang = ALIASES.get(word)
            if word == "pycon":
                page.problems.append(f"{name}:{number}: a pycon fence cannot run; use python")
            if skip and not lang:
                page.problems.append(f"{name}:{skip[0]}: a skip marker must sit right before a python or sql fence")
                skip = None
            fence = opening.group("fence")
            open_fence = (fence[0], len(fence), opening.group("indent"), lang, number, [])
            continue
        if skip:
            page.problems.append(f"{name}:{skip[0]}: a skip marker must sit right before a python or sql fence")
            skip = None
        marker = MARKER.match(line)
        if marker:
            reason = marker.group("reason").strip()
            if not reason:
                page.problems.append(f"{name}:{number}: a skip marker needs a reason")
            skip = (number, reason)
    if open_fence:
        page.problems.append(f"{name}:{open_fence[4]}: a fence is never closed")
    if skip:
        page.problems.append(f"{name}:{skip[0]}: a skip marker must sit right before a python or sql fence")
    return page
