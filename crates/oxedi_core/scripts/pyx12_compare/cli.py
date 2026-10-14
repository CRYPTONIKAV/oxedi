"""The command line."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from .comparison import compare
from .config import IGNORE, OUT, SPECS, VERSIONS
from .maps import pyx12_maps
from .report import failure_text, failures, report
from .spec import load_ignore, load_spec


DESCRIPTION = "Cross-checks an 835 spec against pyx12's maps."


def main(argv=None):
    parser = argparse.ArgumentParser(description=DESCRIPTION)
    parser.add_argument("--version", choices=sorted(VERSIONS), default="5010")
    parser.add_argument("--spec", type=Path, help="spec JSON (default: the built-in 835)")
    parser.add_argument("--spec-patch", type=Path, action="append",
                        help="merge patch applied over --spec (repeatable)")
    parser.add_argument("--map", type=Path, help="pyx12 transaction map")
    parser.add_argument("--dataele", type=Path, help="pyx12 dataele.xml")
    parser.add_argument("--codes", type=Path, help="pyx12 codes.xml")
    parser.add_argument("--ignore", type=Path, default=IGNORE)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--patch", type=Path)
    parser.add_argument("--check", action="store_true",
                        help="exit 1 when what the spec defines disagrees with the map")
    parser.add_argument("--fresh-names", action="store_true",
                        help="propose every occurrence name from the map instead of keeping "
                             "the spec's names")
    args = parser.parse_args(argv)

    version = VERSIONS[args.version]
    if args.spec is None:
        spec_path = SPECS / "835.json"
        patches = [SPECS / name for name in version["patches"]]
    else:
        spec_path, patches = args.spec, []
    patches += args.spec_patch or []
    needs_maps = args.map is None or args.dataele is None or args.codes is None
    maps = pyx12_maps() if needs_maps else None
    map_path = args.map or maps / version["map"]
    ignored = load_ignore(args.ignore, Path(map_path).name)
    comparison = compare(
        load_spec(spec_path, patches),
        map_path,
        args.dataele or maps / "dataele.xml",
        args.codes or maps / "codes.xml",
        ignored,
        args.fresh_names,
    )

    if args.check:
        failed = failures(comparison, ignored)
        for finding in failed:
            print(failure_text(finding), file=sys.stderr)
        if failed:
            print(f"{len(failed)} disagreement(s) between {spec_path.name} and "
                  f"{Path(map_path).name}", file=sys.stderr)
            return 1
        return 0

    report_path = args.report or OUT / f"report-{args.version}.md"
    patch_path = args.patch or OUT / f"patch-{args.version}.json"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    patch_path.parent.mkdir(parents=True, exist_ok=True)
    title = f"{spec_path.name}{''.join(' + ' + Path(p).name for p in patches)} vs pyx12 " \
            f"{Path(map_path).name}"
    report_path.write_text(report(comparison, title, ignored))
    patch_path.write_text(json.dumps(comparison.patch, indent=2) + "\n")
    print(f"report: {report_path}\npatch: {patch_path}")
    return 0
