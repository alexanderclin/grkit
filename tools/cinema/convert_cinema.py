#!/usr/bin/env python3
"""Convert extracted GR IV Cinema look data into GRKit effect definitions.

The input is the output directory of `tools/extract_gr4.py` from
fufuNsoup/GRIIIx-cinema-recipe-log, which reads the Cinema (Yellow) and
Cinema (Green) looks out of your own copy of Ricoh's GR IV firmware
(fwdc248b.bin, GR IV 1.11) using idontknownihil/gr3-film-mods:

    fits/cinema_{yellow,green}.json   look matrix + exact 256-entry curves
    cinema_ref_cinema_{yellow,green}.npy
                                      the curves at all three Color Tones,
                                      indexed [color tone][channel][256]

The conversion is a transplant, not a fit: the extracted curves already use
the 0..16383 domain of a GRKit effect, so they are copied verbatim, and the
float look matrix (given relative to GR III Standard) is quantised to the
Q13 integers a GRKit definition stores. Each look becomes three effects,
one per Color Tone step (the GR IV puts the steps on a slider; a GRKit
effect is a fixed record, so the steps are separate effects).

The GR IV MultiAxial hue sectors are NOT converted: a GRKit effect has no
hue-sector stage. gr3-film-mods FINDINGS section 31 measures that omission
at mean dE2000 1.53 (Yellow) / 1.33 (Green), p95 up to 4.8 on a camera chart.

The output contains Ricoh firmware data. It is for personal use only:
do not commit it, share it, or contribute it upstream. This script contains
no Ricoh data; the data comes only from the firmware you supply.

Usage:
    convert_cinema.py EXTRACT_DIR [OUT_DIR]

OUT_DIR defaults to ./cinema-effects. Each effect lands in
OUT_DIR/<effect-id>/effect.json, ready for `grkit effects publish`.
Requires Python 3.10+ and numpy (the extractor already requires it).
"""

import json
import os
import sys

import numpy as np

LOOKS = {
    "cinema_yellow": {
        "slug": "cinema-yellow",
        "name": "Cinema Yellow",
        "tone": 18,
        "labels": {0: "CYL1", 1: "CYEL", 2: "CYL3"},
    },
    "cinema_green": {
        "slug": "cinema-green",
        "name": "Cinema Green",
        "tone": 19,
        "labels": {0: "CGR1", 1: "CGRN", 2: "CGR3"},
    },
}

# Color Tone index in the extractor's reference curves, the effect ID
# suffix, the display-name suffix, and the attribution note for the step.
VARIANTS = [
    (1, "-v1", "", "Color Tone 2 in GR IV terms (the default; ct index 1)"),
    (0, "-t1-v1", " T1", "Color Tone 1 (weak)"),
    (2, "-t3-v1", " T3", "Color Tone 3 (strong)"),
]

SOURCE = (
    "fwdc248b.bin (GR IV firmware 1.11), extracted with "
    "fufuNsoup/GRIIIx-cinema-recipe-log tools/extract_gr4.py (MIT) over "
    "idontknownihil/gr3-film-mods (MIT)"
)
NOTE = (
    "Look matrix and exact R/G/B tone curves transplanted directly (no fit). "
    "GR IV MultiAxial hue sectors are not represented: GRKit effects have no "
    "hue-sector stage (film-mods FINDINGS S31 measures the sectors at "
    "1.33-1.53 mean dE2000)."
)


def convert(extract_dir: str, out_dir: str) -> list[str]:
    written = []
    for key, look in LOOKS.items():
        fit = json.load(open(os.path.join(extract_dir, "fits", f"{key}.json")))
        matrix = [round(v * 8192) for row in fit["matrix"] for v in row]
        refs = np.load(os.path.join(extract_dir, f"cinema_ref_{key}.npy"))
        assert refs.shape == (3, 3, 256), f"{key}: unexpected refs shape {refs.shape}"
        # The extractor's headline fit is the default Color Tone's curves.
        assert (refs[1] == np.array(fit["presets"][0]["curves"])).all(), (
            f"{key}: default curves disagree with the fit file"
        )
        for ct, id_suffix, name_suffix, ct_note in VARIANTS:
            effect_id = look["slug"] + id_suffix
            name = look["name"] + name_suffix
            assert len(name) <= 16, name
            effect = {
                "schema": "grkit-effect-v1",
                "effect_id": effect_id,
                "name": name,
                "flags": 0,
                "matrix": matrix,
                "curves": [[int(v) for v in refs[ct][ch]] for ch in range(3)],
                "icon": {"label": look["labels"][ct], "shape": "film"},
                "attribution": {
                    "origin": "firmware-transplant",
                    "target": (
                        f"RICOH GR IV {look['name']} (Image Control tone "
                        f"{look['tone']}), {ct_note}"
                    ),
                    "source": SOURCE,
                    "license": "Ricoh firmware data - personal use only, do not redistribute",
                    "note": NOTE,
                },
            }
            directory = os.path.join(out_dir, effect_id)
            os.makedirs(directory, exist_ok=True)
            with open(os.path.join(directory, "effect.json"), "w") as handle:
                json.dump(effect, handle, indent=1)
            written.append(effect_id)
    return written


def main() -> int:
    if len(sys.argv) not in (2, 3):
        print(__doc__.splitlines()[0], file=sys.stderr)
        print("Usage: convert_cinema.py EXTRACT_DIR [OUT_DIR]", file=sys.stderr)
        return 2
    extract_dir = sys.argv[1]
    out_dir = sys.argv[2] if len(sys.argv) == 3 else "cinema-effects"
    for effect_id in convert(extract_dir, out_dir):
        print(effect_id)
    print(f"Wrote {out_dir} - Ricoh data, personal use only; do not commit it.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
