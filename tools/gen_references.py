#!/usr/bin/env python3
"""Generate data/references.json from tools/references_source.txt.

Each colour's hex, and the hex of each pairing, come from data/names_css.json (preferred) or
data/names_xkcd.json, so names and values stay consistent with the naming lists.

    python3 tools/gen_references.py
"""

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "tools/references_source.txt"
OUT = ROOT / "data/references.json"


def load_names():
    names = {}
    for file in ("names_xkcd.json", "names_css.json"):  # CSS last so it wins
        for c in json.loads((ROOT / "data" / file).read_text())["colors"]:
            if not c.get("alias"):
                names[c["name"]] = c["hex"]
    return names


def main():
    names = load_names()
    entries = {}
    errors = []
    for lineno, line in enumerate(SOURCE.read_text().splitlines(), 1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        fields = [f.strip() for f in line.split("|")]
        if len(fields) != 5:
            errors.append(f"line {lineno}: expected 5 fields, got {len(fields)}")
            continue
        name, refs, assoc, uses, pairs = fields
        split = lambda s: [x.strip() for x in s.split(";") if x.strip()]
        if name in entries:
            errors.append(f"line {lineno}: duplicate colour {name!r}")
            continue
        if name not in names:
            errors.append(f"line {lineno}: {name!r} is not in the name lists")
            continue
        pairings = []
        for p in split(pairs):
            if p not in names:
                errors.append(f"line {lineno}: pairing {p!r} is not in the name lists")
            else:
                pairings.append(names[p])
        entries[name] = {
            "hex": names[name],
            "references": split(refs),
            "associations": split(assoc),
            "uses": split(uses),
            "pairings": pairings,
        }
    if errors:
        sys.exit("\n".join(errors))
    OUT.write_text(json.dumps(entries, indent=1, ensure_ascii=False) + "\n")
    print(f"wrote {OUT.relative_to(ROOT)} ({len(entries)} colours)")


if __name__ == "__main__":
    main()
