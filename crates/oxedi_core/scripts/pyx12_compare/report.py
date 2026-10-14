"""The Markdown report and the --check failures."""

from __future__ import annotations

from .config import CATEGORIES


def report(comparison, title, ignored):
    lines = [f"# {title}", ""]
    lines += [
        "Draft patch: a merge patch over the compared spec; review every entry before",
        "applying it. Code lists merge every place a segment is used (see the script).",
        "",
        "| category | match | differs | lacks | note | ignored |",
        "|---|---|---|---|---|---|",
    ]
    for category in CATEGORIES:
        found = [f for f in comparison.findings if f.category == category]
        count = {s: sum(1 for f in found if f.status == s and f.key not in ignored)
                 for s in ("match", "differs", "lacks", "note")}
        skipped = sum(1 for f in found if f.key in ignored)
        lines.append(f"| {category} | {count['match']} | {count['differs']} | "
                     f"{count['lacks']} | {count['note']} | {skipped} |")
    for category in CATEGORIES:
        for status, heading in (("differs", "differs"), ("lacks", "the spec lacks"),
                                ("note", "notes")):
            found = [f for f in comparison.findings
                     if f.category == category and f.status == status and f.key not in ignored]
            if not found:
                continue
            lines += ["", f"## {category}: {heading}", ""]
            lines += [f"- `{f.key}` [{f.loops}] {f.message}" for f in found]
    lines += ["", "## Occurrences generated from the map", ""]
    for name, occurrences in comparison.generated.items():
        lines.append(f"- {name}: " + "; ".join(
            f"`{occ_name}` {occ['segment']}@{occ['pos']}"
            + (f" {occ['qualifier']['codes']}" if occ.get("qualifier") else "")
            + f" {occ['usage'][0].upper()}" + (f" max {occ['max']}" if "max" in occ else "")
            for occ_name, occ in occurrences.items()))
    lines += ["", "## Left out on purpose", ""]
    for name, refs in sorted(comparison.externals.items()):
        size = comparison.codesets.get(name)
        listed = f"codes.xml lists {size}" if size is not None else "not in codes.xml"
        lines.append(f"- external code set `{name}` ({listed}) on {', '.join(sorted(refs))}: "
                     "external code sets are not copied into the spec")
    for key, reason in sorted(ignored.items()):
        lines.append(f"- `{key}`: {reason}")
    if comparison.unmapped:
        lines += ["", "## Unmapped pyx12 loops", ""]
        lines += [f"- {xid}" for xid in comparison.unmapped]
    return "\n".join(lines) + "\n"


def failures(comparison, ignored):
    return [f for f in comparison.findings
            if f.status == "differs" and f.defined and f.key not in ignored]


def failure_text(finding):
    # A message that already names its loop is not prefixed with it again.
    if finding.message.startswith(f"loop {finding.loops}"):
        return f"{finding.key}: {finding.message}"
    return f"{finding.key}: loop {finding.loops}: {finding.message}"
