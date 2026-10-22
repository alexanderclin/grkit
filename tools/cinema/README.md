# GR IV Cinema looks as GRKit effects

RICOH's GR IV has two Image Controls the GR III family never got: Cinema
(Yellow) and Cinema (Green). A GRKit effect is a color matrix plus three
tone curves, and the Cinema looks are exactly that data inside the GR IV
firmware — so they can be transplanted into GRKit effects for the GR III
and GR IIIx, with the camera's own Fn parameter page (saturation, hue,
contrast, and the rest) adjusting them afterwards, per effect, like any
native look.

This directory holds the converter only. It contains no Ricoh data, and
neither may its output be committed or shared: the effect definitions it
writes are Ricoh firmware data, for personal use only.

## What you need

- Your own copy of Ricoh's GR IV firmware update file `fwdc248b.bin`
  (GR IV 1.11), downloaded from Ricoh.
- The extractor from
  [GRIIIx-cinema-recipe-log](https://github.com/fufuNsoup/GRIIIx-cinema-recipe-log)
  (MIT), run once against that file. Follow its `build.sh` as far as the
  "GR IV Cinema data" step, or run its `tools/extract_gr4.py` directly with
  a checkout of [gr3-film-mods](https://github.com/idontknownihil/gr3-film-mods)
  at the commit it pins. You do not need a GR IIIx or the flashing stages.
- Python 3.10+ with numpy.

## Convert and build

```sh
python3 tools/cinema/convert_cinema.py /path/to/extract-output cinema-effects

grkit effects publish --source cinema-effects \
    --firmware /path/to/fwdc239b.bin --out compiled-effects
mkdir package-effects && cp compiled-effects/*.GRF package-effects/
grkit package --build <runtime build> --effects package-effects \
    --guide <guide pdf> --out GRKit-Cinema.zip
```

The converter writes six effects: Cinema Yellow and Cinema Green, each in
three Color Tone steps. On the GR IV the steps are one look and a slider
that re-blends the red and blue curves toward green; a GRKit effect is a
fixed record, so the steps are separate effects (T1 is the weak step, the
plain name is the default middle step, T3 is the strong step).

## Fidelity

The matrix and the 256-entry curves are the camera's exact numbers, copied
(the extracted curves already use the 0..16383 domain a GRKit effect
stores), so there is no fit error. What is missing is the GR IV's
MultiAxial stage: twelve hue sectors with a small matrix each, which a
GRKit effect cannot express. gr3-film-mods FINDINGS section 31 measures
that omission at mean dE2000 1.53 for Cinema (Yellow) and 1.33 for Cinema
(Green), with a camera-chart p95 up to 4.8 — visible on the most saturated
colors, in the same range as GRKit's own film fits, and the tone and color
character are otherwise the genuine look.

Credits: the extraction is fufuNsoup's and idontknownihil's work (MIT);
the Cinema looks themselves are Ricoh's.
