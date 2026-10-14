"""The comparison of loops (ids, parents, triggers) and of the segments each loop holds."""

from __future__ import annotations

from .config import LOOP_MAP
from .findings import element_ref


class LoopsMixin:
    """Loops and the segments per loop."""

    # loops: ids, parents, triggers
    def compare_loops(self):
        spec_loops = self.spec.get("loops", {})
        for loop in self.loops:
            if loop.xid not in LOOP_MAP:
                self.unmapped.append(loop.xid)
        for name in sorted(set(spec_loops) - set(LOOP_MAP.values())):
            self.add("loops", "differs", f"loops:{name}", name,
                     f"spec loop {name!r} has no pyx12 loop in the mapping table",
                     defined=True)
        for loop in self.loops:
            name = LOOP_MAP.get(loop.xid)
            if name is None or loop.kind == "wrapper":
                continue
            ours = spec_loops.get(name)
            if ours is None:
                self.add("loops", "lacks", f"loops:{name}", name,
                         f"pyx12 loop {loop.xid} ({name}) is not in the spec")
                continue
            parent = LOOP_MAP.get(loop.parent) if loop.parent else None
            if ours.get("parent") == parent:
                self.add("loops", "match", f"loops:{name}:parent", name, "parent")
            else:
                self.add("loops", "differs", f"loops:{name}:parent", name,
                         f"loop {name}: parent is {ours.get('parent')!r} in the spec, "
                         f"{parent!r} in pyx12", ours.get("parent"), parent, defined=True)
                self.set_patch(["loops", name, "parent"], parent)
            first = loop.segments[0] if loop.segments else None
            trigger = ours.get("trigger", {})
            if first is None or trigger.get("segment") != first.id:
                self.add("loops", "differs", f"loops:{name}:trigger", name,
                         f"loop {name}: trigger segment is {trigger.get('segment')!r} in "
                         f"the spec, {first.id if first else None!r} in pyx12",
                         trigger.get("segment"), first.id if first else None, defined=True)
                continue
            ok = True
            for position, value in sorted(trigger.get("where", {}).items()):
                element = first.elements.get(int(position))
                allowed = element.codes if element else None
                if allowed is not None and value not in allowed:
                    ok = False
                    self.add("loops", "differs", f"loops:{name}:trigger:{position}", name,
                             f"loop {name}: trigger {element_ref(first.id, int(position))} = "
                             f"{value!r} in the spec; pyx12 allows {list(allowed)}",
                             value, list(allowed), defined=True)
            if ok:
                self.add("loops", "match", f"loops:{name}:trigger", name, "trigger")

    # segments per loop
    def compare_segments(self):
        spec_loops = self.spec.get("loops", {})
        held = {}
        for loop in self.loops:
            name = LOOP_MAP.get(loop.xid)
            if name is None:
                continue
            for seg in loop.segments:
                held.setdefault(name, {}).setdefault(seg.id, []).append(seg.usage)
        for name, segments in held.items():
            ours = spec_loops.get(name)
            if ours is None:
                continue
            listed = [ours.get("trigger", {}).get("segment")]
            listed += [o.get("segment") for o in ours.get("occurrences", {}).values()]
            listed += [ours["end"]] if ours.get("end") else []
            for seg_id, usages in segments.items():
                usage = "R" if "R" in usages else ("S" if "S" in usages else "N")
                if seg_id in listed:
                    status = "match" if usage != "N" else "differs"
                    self.add("segments", status, f"segments:{name}:{seg_id}", name,
                             f"loop {name} holds {seg_id} (pyx12 usage {usage})"
                             if status == "match" else
                             f"loop {name} holds {seg_id}; pyx12 marks it not used",
                             seg_id, usage, defined=status != "match")
                elif usage != "N":
                    self.add("segments", "lacks", f"segments:{name}:{seg_id}", name,
                             f"loop {name} does not hold {seg_id} (pyx12 usage {usage})",
                             None, usage)
            for seg_id in listed:
                if seg_id and seg_id not in segments:
                    self.add("segments", "differs", f"segments:{name}:{seg_id}", name,
                             f"loop {name} holds {seg_id} in the spec; pyx12 does not "
                             "place it there", seg_id, None, defined=True)
