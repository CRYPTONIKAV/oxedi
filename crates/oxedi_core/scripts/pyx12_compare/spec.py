"""Reading a spec and its merge patches, and the RFC 7386 helpers and ignore list."""

from __future__ import annotations

import json
import sys
from pathlib import Path


def merge_patch(target, patch):
    """RFC 7386: objects merge key by key, null deletes, anything else replaces."""
    if not isinstance(patch, dict):
        return patch
    result = dict(target) if isinstance(target, dict) else {}
    for key, value in patch.items():
        if value is None:
            result.pop(key, None)
        else:
            result[key] = merge_patch(result.get(key), value)
    return result


def load_spec(path, patches=()):
    spec = json.loads(Path(path).read_text())
    for patch in patches:
        spec = merge_patch(spec, json.loads(Path(patch).read_text()))
    return spec


def merge_diff(base, target):
    """The RFC 7386 merge patch that turns ``base`` into ``target``."""
    if not isinstance(base, dict) or not isinstance(target, dict):
        return target
    out = {key: None for key in base if key not in target}
    for key, value in target.items():
        if key not in base:
            out[key] = value
        elif base[key] != value:
            both = isinstance(base[key], dict) and isinstance(value, dict)
            out[key] = merge_diff(base[key], value) if both else value
    return out


def load_ignore(path, map_name):
    """{key: reason} for the entries that apply to ``map_name``."""
    if not Path(path).exists():
        return {}
    entries = json.loads(Path(path).read_text()).get("ignore", [])
    out = {}
    for i, entry in enumerate(entries):
        reason = (entry.get("reason") or "").strip()
        if not entry.get("key") or not reason:
            sys.exit(f"{path}: ignore[{i}] needs a non-empty \"key\" and \"reason\"")
        if entry.get("map") in (None, map_name):
            out[entry["key"]] = reason
    return out
