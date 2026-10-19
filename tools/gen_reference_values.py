#!/usr/bin/env python3
"""Generate crates/color-core/tests/fixtures/reference_conversions.json with colour-science.

colour-science is an independent implementation used only to produce test expectations:
    python3 -m venv .venv && .venv/bin/pip install colour-science
    .venv/bin/python tools/gen_reference_values.py
"""

import json
from pathlib import Path

import colour
import numpy as np

HEXES = [
    "#000000", "#FFFFFF", "#808080", "#FF0000", "#00FF00", "#0000FF",
    "#FFFF00", "#00FFFF", "#FF00FF", "#87CEEB", "#8B4513", "#FFA500",
    "#4B0082", "#2E8B57", "#C0C0C0", "#123456", "#FEDCBA", "#7F7F7F",
]

OUT = Path(__file__).resolve().parent.parent / "crates/color-core/tests/fixtures/reference_conversions.json"


def srgb_derived():
    """sRGB with the matrix derived from its primaries (as in CSS Color 4 / Lindbloom),
    not the 4-decimal IEC 61966-2-1 matrix, which shifts CIELAB by ~0.02."""
    space = colour.RGB_COLOURSPACES["sRGB"].copy()
    space.use_derived_transformation_matrices(True)
    return space


def main():
    srgb = srgb_derived()
    rows = []
    for hex_value in HEXES:
        rgb = np.array([int(hex_value[i : i + 2], 16) / 255 for i in (1, 3, 5)])
        linear = colour.models.eotf_sRGB(rgb)
        xyz = colour.RGB_to_XYZ(rgb, srgb, apply_cctf_decoding=True)
        lab = colour.XYZ_to_Lab(xyz, colour.CCS_ILLUMINANTS["CIE 1931 2 Degree Standard Observer"]["D65"])
        oklab = colour.XYZ_to_Oklab(xyz)
        oklch = colour.Lab_to_LCHab(oklab)
        rows.append(
            {
                "hex": hex_value,
                "linear": linear.round(6).tolist(),
                "xyz": xyz.round(6).tolist(),
                "lab": lab.round(4).tolist(),
                "oklab": oklab.round(6).tolist(),
                "oklch": oklch.round(6).tolist(),
            }
        )
    doc = {"source": f"colour-science {colour.__version__}", "colors": rows}
    OUT.write_text(json.dumps(doc, indent=1) + "\n")
    print(f"wrote {OUT} ({len(rows)} colors)")


if __name__ == "__main__":
    main()
