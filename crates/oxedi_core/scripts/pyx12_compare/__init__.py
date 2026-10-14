"""The spec cross-check against pyx12's maps, one module per concern.

- ``config``: paths, the compared versions, the loop mapping table.
- ``maps``: reading pyx12's transaction map, data elements and code sets.
- ``spec``: reading a spec and its merge patches, RFC 7386 helpers, the ignore list.
- ``findings``: the comparison's records and the helpers that normalise what is compared.
- ``loops``: the loop and per-loop segment comparisons.
- ``occurrences``: occurrences generated from the map, matched with the spec's, compared.
- ``elements``: element usage, types, lengths and codes, and the draft definitions.
- ``comparison``: the comparison's state and the order of its passes.
- ``report``: the Markdown report and the ``--check`` failures.
- ``cli``: the command line.
"""

from .cli import main
from .comparison import Comparison, compare
from .config import SPECS, VERSIONS
from .maps import load_map, map_position
from .spec import merge_diff, merge_patch

__all__ = [
    "Comparison",
    "SPECS",
    "VERSIONS",
    "compare",
    "load_map",
    "main",
    "map_position",
    "merge_diff",
    "merge_patch",
]
