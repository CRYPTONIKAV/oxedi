"""The comparison of a spec with a pyx12 map: its state and the order of the passes."""

from __future__ import annotations

from .elements import ElementsMixin
from .findings import Finding
from .loops import LoopsMixin
from .maps import load_codesets, load_dataele, load_map
from .occurrences import OccurrencesMixin


class Comparison(LoopsMixin, OccurrencesMixin, ElementsMixin):
    def __init__(self, spec, loops, dataele, codesets, ignored=None, fresh_names=False):
        self.spec = spec
        self.loops = loops
        self.dataele = dataele
        self.codesets = codesets
        # Ignored finding keys: a code list kept apart from the map on purpose
        # is not narrowed per occurrence either.
        self.ignored = ignored or {}
        # Propose every occurrence name from the map instead of keeping the
        # names the spec already gives its occurrences.
        self.fresh_names = fresh_names
        self.findings: list[Finding] = []
        self.patch: dict = {}
        self.unmapped: list[str] = []
        self.externals: dict[str, set[str]] = {}
        # Spec loop name -> {occurrence name: occurrence} generated from the map.
        self.generated: dict[str, dict] = {}

    def add(self, category, status, key, loops, message, spec_value=None, map_value=None,
            defined=False):
        self.findings.append(
            Finding(category, status, key, loops, message, spec_value, map_value, defined)
        )

    def set_patch(self, path, value):
        node = self.patch
        for part in path[:-1]:
            node = node.setdefault(part, {})
        node[path[-1]] = value

    def run(self):
        self.compare_loops()
        self.compare_segments()
        self.compare_occurrences()
        self.compare_elements()
        return self


def compare(spec, map_path, dataele_path, codes_path, ignored=None, fresh_names=False):
    return Comparison(
        spec, load_map(map_path), load_dataele(dataele_path), load_codesets(codes_path),
        ignored, fresh_names,
    ).run()
