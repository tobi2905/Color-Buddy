#!/usr/bin/env python3
"""Validate data/references.json: schema, hex values, duplicate keys and entry count.

    python3 tools/validate_references.py   (exit code 1 on any problem; runs in CI)
"""

import json
import re
import sys
from pathlib import Path

PATH = Path(__file__).resolve().parent.parent / "data/references.json"
MIN_ENTRIES = 200
HEX = re.compile(r"^#[0-9A-F]{6}$")
LIST_FIELDS = ("references", "associations", "uses")
# "uses" is exactly [industry or brand type, design role, theme or occasion]; the middle slot
# comes from this list so the three always differ in kind (decided at review checkpoint 4).
DESIGN_ROLES = {
    "backgrounds", "surfaces", "body text", "headings", "buttons", "links", "highlights",
    "accents", "borders", "icons", "illustrations", "charts", "logos", "packaging", "badges",
    "success states", "error states", "warning states", "dark mode backgrounds",
}
BASIC_TERMS = ["red", "orange", "yellow", "green", "blue", "purple", "pink", "brown", "black", "white", "grey"]


def reject_duplicates(pairs):
    keys = [k for k, _ in pairs]
    dupes = sorted({k for k in keys if keys.count(k) > 1})
    if dupes:
        raise ValueError(f"duplicate keys: {', '.join(dupes)}")
    return dict(pairs)


def main():
    problems = []
    try:
        data = json.loads(PATH.read_text(), object_pairs_hook=reject_duplicates)
    except ValueError as e:
        sys.exit(f"{PATH.name}: {e}")
    if not isinstance(data, dict):
        sys.exit(f"{PATH.name}: top level must be an object keyed by colour name")

    for name, entry in data.items():
        where = f"{name!r}"
        if name != name.strip().lower() or not name:
            problems.append(f"{where}: keys must be lowercase names")
        if not isinstance(entry, dict):
            problems.append(f"{where}: must be an object")
            continue
        expected = {"hex", *LIST_FIELDS, "pairings"}
        if set(entry) != expected:
            problems.append(f"{where}: fields must be exactly {sorted(expected)}, got {sorted(entry)}")
        if not isinstance(entry.get("hex"), str) or not HEX.match(entry["hex"]):
            problems.append(f"{where}: hex must be #RRGGBB (uppercase), got {entry.get('hex')!r}")
        for field in LIST_FIELDS:
            values = entry.get(field)
            if not isinstance(values, list) or not values:
                problems.append(f"{where}: {field} must be a non-empty list")
            elif not all(isinstance(v, str) and v.strip() for v in values):
                problems.append(f"{where}: {field} must contain non-empty strings")
            elif len(set(values)) != len(values):
                problems.append(f"{where}: {field} has repeated items")
        uses = entry.get("uses")
        if isinstance(uses, list) and all(isinstance(u, str) for u in uses):
            if len(uses) != 3:
                problems.append(f"{where}: uses must be [industry, design role, theme], got {len(uses)} items")
            elif uses[1] not in DESIGN_ROLES:
                problems.append(f"{where}: second use {uses[1]!r} is not a design role ({', '.join(sorted(DESIGN_ROLES))})")
            elif uses[0] in DESIGN_ROLES or uses[2] in DESIGN_ROLES:
                problems.append(f"{where}: only the second use may be a design role")
        pairings = entry.get("pairings")
        if not isinstance(pairings, list) or not pairings:
            problems.append(f"{where}: pairings must be a non-empty list")
        elif not all(isinstance(p, str) and HEX.match(p) for p in pairings):
            problems.append(f"{where}: pairings must be #RRGGBB values")
        elif entry.get("hex") in pairings:
            problems.append(f"{where}: a colour can't pair with itself")

    missing = [t for t in BASIC_TERMS if t not in data]
    if missing:
        problems.append(f"family entries missing for: {', '.join(missing)}")
    if len(data) < MIN_ENTRIES:
        problems.append(f"{len(data)} entries; at least {MIN_ENTRIES} required")

    if problems:
        sys.exit("\n".join(problems))
    print(f"{PATH.name}: {len(data)} entries OK")


if __name__ == "__main__":
    main()
