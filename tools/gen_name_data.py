#!/usr/bin/env python3
"""Generate data/names_*.json from upstream sources.

Usage:
    curl -sSfLo xkcd_rgb.txt https://xkcd.com/color/rgb.txt
    curl -sSfLo css-color.html https://www.w3.org/TR/css-color-4/
    python3 tools/gen_name_data.py xkcd_rgb.txt css-color.html

Only the standard library is used.
"""

import json
import re
import sys
from pathlib import Path

DATA = Path(__file__).resolve().parent.parent / "data"

BASIC_TERMS = [
    "red", "orange", "yellow", "green", "blue", "purple",
    "pink", "brown", "black", "white", "grey",
]
# Variants whose survey colour clearly belongs to another basic term.
BASIC_EXCLUDE = {"pale grey"}  # #fdfdfe is white

# XKCD names we don't want to show to users.
XKCD_BLOCKLIST = re.compile(r"shit|poo|puke|vomit|diarrhea|booger|barf|snot")

# Vocabulary for splitting CSS keywords ("lightgoldenrodyellow" -> "light goldenrod yellow").
CSS_WORDS = """
alice almond antique burlywood firebrick aqua aquamarine azure beige bisque black blanched blue blush brick brown
burly cadet chartreuse chiffon chocolate coral cornflower cornsilk cream crimson cyan dark deep
dim dodger drab fire floral forest fuchsia gainsboro ghost gold golden goldenrod gray green grey
honeydew hot indian indigo ivory khaki lace lavender lawn lemon light lime linen magenta maroon
marine medium midnight mint misty moccasin navajo navy old olive orange orchid pale papaya peach
peru pink plum powder puff purple rebecca red rose rosy royal saddle salmon sandy sea seashell
sienna silver sky slate smoke snow spring steel tan teal thistle tomato turquoise violet wheat
whip white wood yellow
""".split()

# When two CSS keywords share a hex value, prefer this one for naming.
CSS_PREFERRED = {"cyan", "magenta"}


def split_css(keyword: str) -> str:
    """Split a CSS keyword into words, using as few vocabulary words as possible."""
    best = {0: []}
    for end in range(1, len(keyword) + 1):
        for start in range(end):
            word = keyword[start:end]
            if start in best and word in CSS_WORDS:
                cand = best[start] + [word]
                if end not in best or len(cand) < len(best[end]):
                    best[end] = cand
    if len(keyword) not in best:
        raise ValueError(f"cannot split CSS keyword {keyword!r}")
    return " ".join(best[len(keyword)])


def read_xkcd(path):
    colors = []
    for line in Path(path).read_text().splitlines():
        if line.startswith("#") or not line.strip():
            continue
        name, hex_value = [p.strip() for p in line.split("\t")[:2]]
        # A few survey names are glued ("lightblue"); spell them like the rest.
        name = re.sub(rf"^(light|dark)(?=(?:{'|'.join(BASIC_TERMS)})$)", r"\1 ", name)
        colors.append({"name": name, "hex": hex_value.upper()})
    return colors


def read_css(path):
    html = Path(path).read_text()
    start = html.index('class="named-color-table')
    table = html[start : html.index("</table>", start)]
    rows = re.findall(r'id="valdef-color-([a-z]+)">\1</dfn>\s*<td>(#[0-9a-f]{6})', table)
    if len(rows) != 148:
        raise ValueError(f"expected 148 CSS named colors, found {len(rows)}")
    return rows


def write(name, source, license_, colors):
    doc = {"source": source, "license": license_, "colors": colors}
    out = DATA / name
    out.write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n")
    print(f"wrote {out.relative_to(DATA.parent)} ({len(colors)} colors)")


def main(xkcd_path, css_path):
    DATA.mkdir(exist_ok=True)
    xkcd = read_xkcd(xkcd_path)
    by_name = {c["name"]: c["hex"] for c in xkcd}

    # Anchors for each basic term: every survey name whose head noun (last word) is that term,
    # e.g. "reddish brown" and "rust brown" are brown, "royal purple" is purple.
    basic = []
    for c in xkcd:
        head = c["name"].split()[-1]
        if head in BASIC_TERMS and c["name"] not in BASIC_EXCLUDE and not XKCD_BLOCKLIST.search(c["name"]):
            basic.append({"name": head, "hex": c["hex"], "anchor": c["name"]})
    write(
        "names_basic.json",
        "11 basic colour terms; anchors are all XKCD colour survey names whose last word is the "
        "term, e.g. 'reddish brown' for brown (https://xkcd.com/color/rgb.txt)",
        "CC0-1.0",
        basic,
    )

    kept = [c for c in xkcd if not XKCD_BLOCKLIST.search(c["name"])]
    write(
        "names_xkcd.json",
        f"XKCD colour survey (https://xkcd.com/color/rgb.txt); {len(xkcd) - len(kept)} "
        "crude names removed",
        "CC0-1.0",
        kept,
    )

    css = []
    seen_hex = {}
    for keyword, hex_value in read_css(css_path):
        hex_value = hex_value.upper()
        entry = {"name": split_css(keyword), "css": keyword, "hex": hex_value}
        css.append(entry)
        seen_hex.setdefault(hex_value, []).append(entry)
    # Mark aliases (aqua/cyan, fuchsia/magenta, gray/grey spellings) so naming shows one name.
    for entries in seen_hex.values():
        if len(entries) < 2:
            continue
        preferred = next(
            (e for e in entries if e["css"] in CSS_PREFERRED or "grey" in e["css"]),
            entries[0],
        )
        for e in entries:
            if e is not preferred:
                e["alias"] = True
    write(
        "names_css.json",
        "CSS named colors (https://www.w3.org/TR/css-color-4/#named-colors)",
        "W3C Document License",
        css,
    )


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    main(sys.argv[1], sys.argv[2])
