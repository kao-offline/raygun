# Semantic region design

Use this when a concrete question needs semantic measurement or a local mask
needs coordinate/coverage review. Routine visual grading does not require a
region map. Define only regions relevant to the current question; add others
when a new issue calls for them.

Define normalized coordinates only after orientation, crop, and perspective are
accepted. One region definition then works across previews and full-resolution
exports with that same framing. If geometry changes, redraw the regions. A
reference with different framing needs separately placed regions for the same
semantic material; identical numbers do not make the pixels comparable.
Coordinates use the top-left origin and the range 0–1.

Treat the pixel array of the accepted, correctly oriented render as the sole
coordinate authority. Analysis regions, landmark output, review crops, and
drawn XMP masks must all be converted into that same frame; source-file
dimensions before EXIF orientation, GUI screenshots, and hand-resized crops are
not interchangeable coordinate systems. Before measuring candidates or
building a local-light XMP, inspect one overlay on the accepted render and
confirm both the semantic samples and the intended mask center/extent. After a
local stage is rendered, toggle it against its parent and verify that the
visible change is centered on the same semantic target. A plausible normalized
number is not evidence of alignment.

## JSON format

```json
{
  "sky": {
    "kind": "rects",
    "items": [[0.05, 0.04, 0.38, 0.25], [0.67, 0.04, 0.95, 0.25]]
  },
  "ordinary_face_skin": {
    "kind": "ellipses",
    "items": [[0.50, 0.34, 0.035, 0.03]]
  },
  "visible_limbs": {
    "kind": "ellipses",
    "items": [[0.35, 0.47, 0.035, 0.02], [0.75, 0.42, 0.05, 0.025]]
  }
}
```

Rectangles are `[x0, y0, x1, y1]`. Ellipses are `[cx, cy, rx, ry]`. `analyze_photo.py` validates every coordinate.
Copy [regions-example.json](regions-example.json) as a starting point, then move or replace every region to match the actual photograph. The example coordinates are illustrative only.
For the sweet-portrait audit, keep the canonical names
`ordinary_face_skin`, `whole_subject`, and `nearby_environment`. Add
`visible_limbs`, `textured_white`, `black_anchor`, `cheek_vitality`, and `lips`
only when those materials are visibly present and useful. Add or remove actual
shapes to fit the frame.
Use `bright_diffuse` for the brightest non-specular cloth/wall face and
`attention_competitor` for a distracting bright or saturated setting area when
relevant. These supplement the midtone-white sample and nearby-environment
lighting comparator. A dark chest patch does not test a bright collar; keep
separate planes or materials separate when their contrast is under review.
Combining shadowed cloth with a flat bright panel can hide the panel's lost
variation behind a broad overall span.

## Sampling rules

- Select regions that represent a material or semantic role, not arbitrary large areas.
- Avoid boundaries, specular highlights, deep occlusion, text, watermarks, and unrelated objects unless they are the thing being measured.
- Use two or more sky patches when clouds or gradients occupy part of the sky.
- Sample the face and materially visible hands or limbs separately before
  optionally combining them. Different illumination makes one skin average
  misleading. For skin optimization, use representative forehead/cheek/jaw
  samples and avoid lips, makeup, specular highlights, deep occlusion, hair, and
  background edges unless one of those is the explicit diagnostic target.
- When red/orange correction or blood-color vitality is under review, define lips and cheeks as separate diagnostic regions rather than folding them into the ordinary face sample. Add visible ears and fingertips when useful. This keeps healthy accents from being mistaken for diffuse redness and reveals when broad desaturation has erased them.
- For a smiling mouth, select clean lip tissue and exclude teeth, oral cavity
  and surrounding skin. If the lip is too small for representative sampling,
  keep an unannotated face-context comparison and make a visual judgment.
- When relevant, sample dark material such as hair or deep clothing and a
  textured white such as lace or fabric. For a person-lighting change, put the
  dark sample on the affected person; a black doorway cannot show that the hair
  turned gray or lost shape. Include a representative run of folds or strands
  and their lit planes: a uniformly black patch measures density but cannot
  establish retained detail. Keep differently lit dark materials separate.
  Use `black_anchor`, names beginning `black_anchor_`, or the audit's explicit
  `--dark-region` for other names.
- For ambient-night or flash/mixed portraits, sample an unlit background anchor
  separately from practical lights and the subject. Do not use a lamp hotspot as
  the nearby-environment comparator; inspect it separately for motivated color,
  highlight shape, and falloff.
- When subject separation matters, add a broad subject region and a nearby
  environment region under comparable light. Avoid using sky, a specular patch,
  or an arbitrarily dark object as the comparator merely to produce a favorable
  difference.
- Make `whole_subject` follow the actual figure rather than one oversized ellipse
  that includes a dark doorway, sky, or bright platform. A union of simple
  overlapping shapes is usually sufficient for measurement; verify its overlay
  against the full figure. An invalid subject region can create a false
  underexposure verdict just as an invalid skin region creates a false complexion
  verdict.
- For vegetation, sample both foreground and background when depth haze or blur differs.
- For water, avoid isolated sun glints unless the reference intentionally emphasizes them.
- If the reference is lower resolution, compare broad tone/color only. Do not chase its lost microcontrast.

## Locate semantic regions reliably

Choose the cheapest reliable locator available in the current environment; do
not bind the workflow to one operating system or make an optional detector a
project dependency. A platform-native face/landmark API is useful when it
returns stable original-image coordinates—for example, Vision on macOS. On
other systems, prefer an available lightweight local face or body landmark
detector with the same coordinate property. Use darktable's object segmentation
when an edit needs an object boundary, not as a replacement for facial
landmarks or ordinary-skin judgment.

If no convenient local locator is available, use the multimodal model to place
regions directly on the full source image. In every backend, normalize EXIF
orientation first, convert detector coordinates to the accepted full-resolution
framing, render an overlay, and inspect it before measuring. Preview screenshots
with letterboxing, UI chrome, or unknown scaling are not a coordinate system.
Landmarks locate candidate face/limb areas; visual semantic judgment still
separates ordinary skin from eyes, lips, makeup, hair, occlusion, and highlights.

## Reading the measurements

- Use luma to decide where brightness differs. Use HSV only to decide the direction of color change; HSV hue becomes unstable at very low saturation.
- When the image and brief select the sweet-portrait / 糖水人像 profile,
  inspect the ordinary-skin histogram, P25/P50/P75, and the proportion at or
  above the display midpoint. Check highlight headroom and the crop before
  interpreting it; the intended finish comes from the portrait brief.
- Interpret hue circularly. Values near 0° and 360° are adjacent, not far apart.
- Prefer region trends across two or three samples over a single exact number.
- If global statistics match but the image feels wrong, inspect subject/background weighting. A large sky can hide an underexposed face in the global average.
- If a reference has elevated P01 but the target's black hair still looks healthy, do not force all shadows to match P01. Preserve believable local black.
- Do not force skin toward a universal hue or luminance number. Read face, hands, and limbs relative to their illumination, nearby neutrals, channel clipping, and retained warm-red variation; confirm low-chroma hue measurements visually.

## Skin measurement validity gate

Before passing a region to `--skin-region`, inspect one generated region map
and its sample-detail crops on the accepted geometry. Use a
few small patches of genuinely ordinary skin—such as clear forehead, jaw, or
unmade-up limb areas—instead of one broad face ellipse. A named `face_skin`
region is not evidence merely because its JSON is valid: discard or redraw it
if it lands on hair, eyes, lips, strong makeup, clothing, background, deep
occlusion, or isolated highlights.

Keep ordinary-skin, cheek-vitality, and lip regions separate. Use ordinary-skin
regions for fairness conclusions; cheek/lip regions are comparison checks for
whether cleanup erased blood color. Statistics from an unverified or mixed
sample must not influence the XMP decision.

Landmark detection establishes location, not sample purity. A visibly bimodal
or abnormally long-tailed ordinary-skin distribution—such as a very low L25
beside a bright L50/P75, or an
L90 far beyond the visible ordinary skin—is evidence to revisit the overlay, not
evidence that the face needs more exposure. If mixed, rebuild the frozen region
and every compared measurement.

Use clean patches covering the ordinary planes actually visible. With fewer
than 128 analyzed face pixels, increase analysis resolution before interpreting
percentiles, or use face-context evidence for a genuinely tiny or obscured face.
Frame coverage alone is not sample validity: a wide portrait can have hundreds
of clean face pixels but a very small percentage. Once their placement and
context are checked, reuse that decision instead of enlarging regions or
repeating sample boards merely to increase coverage.

Freeze the region JSON after that inspection. Reuse the map for aligned color
and tone variants; recheck it after geometry changes or implausible measurements.
If a region is invalid, correct it and regenerate all compared statistics with
the same definition. A sample represents a material, independent of which
candidate it favors.

## Review outputs

For different photographs or a reference, `build_region_review_board.py` accepts
`--candidate-regions NAME=regions.json` for each separately framed candidate.
Use shared semantic names with separately placed rectangles for face/figure and
recurring materials; missing materials display as not present. This creates one
useful set comparison without reusing coordinates across photographs. Native
tiles remain restricted to aligned renders of the same photograph.

Pass material candidates to `analyze_photo.py` with explicit
`--parent child=parent` relationships. Its `comparisons.json` and report record
regional luminance/saturation/hue deltas and visible-skin region relationships.
They reveal uneven or unexpectedly broad movement; they do not define fixed
acceptable skin values.

For a fair-skin round, pass each verified ordinary-skin sample with
`--skin-region NAME`. The script adds L*/a*/b*/C* percentile tables,
lightness-banded color statistics, and `skin-lab-<name>-overlay.png`. Compare the
same region across source, parent, candidate, and approved anchor when one
exists. Use cheek/lip Lab output only as a separate vitality diagnostic, never
as the ordinary-skin target.

For an exposure or local-light round, add repeated `--histogram-region face`,
`--histogram-region hands`, or the actual region names. The script writes
`histogram-region-<name>-overlay.png` so the source and candidates can be
compared on the same semantic pixels. Use `--histogram-region all` only when all
defined regions are genuinely useful; a few deliberate overlays are easier to
read.

For an explicit hierarchy question, add
`--region-pair subject_vs_environment=whole_subject,nearby_environment`. The
report records both regions and their mean/P50 separation plus the change from
the parent. Read [tonal-hierarchy.md](tonal-hierarchy.md) for interpretation;
this file defines valid regions, not target separation values.

Use the same JSON with `build_region_review_board.py` to create aligned
full-frame and local candidate crops. For portraits, define the face and each
materially visible skin region separately so none is hidden inside a combined
average. Add separate lips and cheek crops after red/orange edits or when
vitality is in question. Add only relevant material rows; a concise board that
is actually inspected is more useful than an exhaustive one.
The board preserves region bounds and gives tall contexts more row height.
Use a rectangle around the entire figure to compare face/limb relationships;
a face strip or a small person in a contact sheet cannot establish them.

For small or multi-item regions, use the sample-detail mode rather than trusting
the combined crop:

```bash
uv run "$PHOTO_XMP_SKILL_DIR/scripts/build_region_review_board.py" \
  --candidate parent=previews/skin-parent.jpg \
  --candidate final=previews/final.jpg \
  --regions analysis/regions.json \
  --region ordinary_face_skin --sample-detail \
  --output analysis/ordinary-skin-review.jpg
```

It keeps an unannotated context crop, then magnifies each item with labels in
the row margin and surroundings shaded. No outline or badge covers tiny sample
pixels. These magnified rows establish sample purity, not native detail. Inspect
every item for facial features, hair, contour, clothing, background, occlusion,
and isolated highlights. Use `--item-padding` only to change the viewing context;
it does not change the measured region. A clean combined-looking crop cannot
excuse one contaminated sample.

For mask continuity, use a separate review-region JSON covering whole edited
surfaces; ordinary-skin measurement samples are too small. When a context board
would shrink a narrow feature, `--native-tiles` writes overlapping PNGs with
one source pixel per tile pixel, plus an index of covered coordinates:

```bash
uv run "$PHOTO_XMP_SKILL_DIR/scripts/build_region_review_board.py" \
  --candidate parent=full-size/parent.jpg --candidate final=full-size/final.jpg \
  --regions analysis/coverage-regions.json --region affected_surface \
  --native-tiles --output analysis/native-review
```

Inputs must have identical full-resolution dimensions. The output is a new
directory; view its actual tiles at native scale. Use only relevant surfaces or
cue regions from `audit_mask.py`, not every pixel of an unchanged background.
This mode complements the quick full-frame judgment; it does not replace it.
For a runtime mask, the masking guide's `audit_mask.py --review-dir` combines
these tiles with a grayscale weight column in the same call.

The analyzer and board convert embedded ICC color to sRGB and normalize EXIF
orientation. Untagged RGB is explicitly assumed sRGB. The analyzer defaults to
a 1600-pixel long edge and records image/region hashes and dimensions; adjust
`--max-edge` when sample resolution requires it. Aspect mismatch is rejected,
but same-aspect crop or perspective changes still need visual registration.
For a differently framed reference use `--reference-regions` with the same
semantic names at separately placed coordinates. For repeated measurements
after the overlay is accepted, `--no-plots` avoids redundant plot generation.
