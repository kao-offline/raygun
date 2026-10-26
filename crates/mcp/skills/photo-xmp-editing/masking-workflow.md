# Masking workflow

Choose and refine masks for the requested edit.

## Choose the mask by the required boundary

- **The surrounding surfaces can share the light:** use a broad, strongly
  feathered ellipse or gradient, with its transition hidden in compatible
  surrounding space.
- **Pixel property matters more than identity:** use a parametric lightness or
  color mask. Add a loose drawn region only to limit its spatial reach.
- **The edit must stop on the subject/object edge:** use darktable-native
  prompted segmentation, or a deliberate path/brush for a small simple area.
- **An earlier XMP already has a good mask:** inspect and clone its group with
  `--reuse-mask`; do not regenerate it without a reason.

Choose reach from the surfaces that should change, not from a default preference
for broad or precise masks. A correct person silhouette does not isolate skin
or protect dark clothing. Decide the tonal/material response separately using
[tonal-hierarchy.md](tonal-hierarchy.md). Hair, translucent cloth, motion blur
and backlight can favor a broad light field. A narrow limb surrounded by independently lit
surfaces may instead need separate fitted shapes or a path, keeping gaps and
neighboring material stable. Inspect the full light footprint, including any
pooled brightness behind the subject; a smooth edge alone does not make it
natural. A requested subject-only or fixed-strength experiment keeps its
boundary and adjustment.

When authoring path/brush JSON, use initialized Bézier `ctrl1`/`ctrl2` handles.
Some CLI encoders leave omitted handles at `(-1,-1)`; headless darktable export
does not perform the GUI's automatic initialization, which can send curves
toward the frame corner. For polygon segments, set both handles to that point's
`corner` and `state: 2`; smooth curves need their actual handles. Inspect the
rendered contour before tuning color or light. This concerns newly authored
paths, not already initialized AI paths or inherited masks.

A drawn-plus-parametric intersection still follows pixel-property edges inside
the drawn boundary. Dark-range protection can also exclude shaded skin, joints
or cloth folds inside a correctly selected person. When that creates uneven
light, use a smooth tonal response or separate material reach; changing the AI
outline will not repair a tonal exclusion. A strong exposure lift can outline
backlit hair, wires, architecture, foliage, or a bright background even when the
drawn ellipse is wide and feathered. For open-ended
development, resolve structural lighting in the tonal parent, which may include
local recovery; do not interpret upstream as necessarily global. Choose strength
from actual transitions and material headroom. A prescribed exposure test keeps
its requested adjustment.

## Prompt darktable's active mask model

Inspect `photo-xmp doctor --help`. Where supported, run
`doctor --require-native-ai` and require a zero exit with
`native_subject_mask.available=true`; otherwise confirm availability and a real
`mask subject` inference. A discovered library or compiled helper does not prove
runtime/model loading. The feature follows the user's darktable 5.6+
installation: it is absent when AI support is not compiled, AI is disabled, the
active model is unset/incomplete, the compatible toolchain cannot link the
bridge, or the helper cannot load the runtime's private dependencies.

Start with one foreground point well inside the intended subject:

    photo-xmp mask subject --source photo.jpg --foreground 0.50 0.42 \
      --output subject.json --alpha subject-ai-alpha.png \
      --preview subject-preview.jpg

Review semantic coverage before edge tuning. Decide which carried or worn
objects belong to the edit; a person prediction may omit them. Add foreground
points well inside required missing parts or objects, background points for
included surroundings/gaps, or a tight `--box LEFT TOP RIGHT BOTTOM` when the
active SAM model supports it. Coordinates are normalized. Use threshold, cleanup,
smoothing, and `--feather-px` only on a substantially correct selection; they
cannot rescue the wrong semantic object.

The preview marks include points in green, exclude points in red, and an
optional box in yellow. The colored overlay is the selected probability mask.
Use it to inventory the surfaces the adjustment should reach, including small
or disconnected parts. Dark fabric, joints, folds and cast shadows still belong
to the object; their darkness alone does not justify a hole in broad subject
light. Distinguish them from actual background gaps and occluding objects.
Preserve the JSON's source/model/prompt settings when refining the prediction;
record manual path changes separately from the original inference metadata.

## Apply image-guided refinement

Pass the reviewed JSON as `--subject-mask` to the module that needs it. The CLI
verifies the source SHA-256 and embeds outer contours plus holes as editable
paths/groups. Three controls act at different stages:

| Control | Effect |
|---|---|
| `mask subject --feather-px` | Geometric feather on the vectorized paths; it does not follow image edges. |
| `edit … --mask-feathering-radius` | darktable **Mask refinement → Feathering radius**: image-guided filtering of the module's mask to better follow nearby image edges. |
| `edit … --mask-blur-radius` | Ordinary Gaussian mask blur; it smooths independently of image edges. |

Confirm the selected module's flags with its `--help`. The refinement is a
module blend operation, not another AI inference. It can also refine compatible
drawn/parametric masks. Its radius is a darktable parameter, not interchangeable
with `--feather-px` or a guaranteed number of final-image pixels; see the
[darktable refinement controls](https://docs.darktable.org/usermanual/development/en/darkroom/masking-and-blending/masks/refinement-controls/).

Supply a mask definition (`--subject-mask`, `--reuse-mask`, drawn shapes, or
parametric channels) when setting refinement. In the current direct-edit CLI,
refinement flags alone do not update an inherited blend without a mask
definition. A successful command is therefore not proof that a radius changed.

### Compare refinement without changing the edit

Keep the same source, parent, segmentation JSON, path feather/smoothing, module
instance, exposure, opacity, blend color space, and other refinement settings.
Include a radius-zero control; keep Gaussian blur at zero when isolating guided
refinement. Example from implicit defaults for a requested +1.2 EV test:

```bash
for radius in 0 5 10; do
  photo-xmp edit exposure --source photo.jpg --subject-mask subject.json \
    --exposure 1.2 --black 0 --compensate-exposure-bias 0 \
    --mask-colorspace rgb_scene --mask-opacity 100 \
    --mask-feathering-radius "$radius" --mask-blur-radius 0 \
    --name "subject exposure" --mask-name subject \
    --output "subject-r${radius}.xmp"
done
```

These EV/radius values illustrate one comparison, not a preset or required
ladder. Use the requested adjustment and bracket the radius around the observed
edge error. Select the smallest radius that gives an acceptable transition;
larger radii can leak into the background or weaken narrow features and holes.
For an existing edit, preserve its accepted parent. The direct module editor
patches an existing instance; do not accidentally turn a global exposure
correction into a masked one.

Use `audit_xmp_axis.py` with `--vary-operation exposure` and
`--vary-field blendop_params` (plus `--vary-name` when needed) to catch changes
outside the selected blend. It compares the whole blend blob, not individual
refinement fields: a pass alone cannot prove that opacity or another blend
setting stayed fixed. Keep those values identical in the authored commands.

## Inspect the runtime mask and actual adjustment

The AI preview precedes path conversion and module refinement. Small false
holes, contour notches or loops can survive conversion; guided filtering and
parametric intersections can also change coverage. Compare the rendered edit
with its parent at the intended strength. For an unexplained patch, toggle the
relevant module while holding other processing fixed; use a radius-zero version
when refinement is implicated. Correct coordinate mismatches first using
[region-design.md](region-design.md).

Review the whole light footprint at viewing size. For consequential local
lighting, name the visible surfaces it reaches before cropping, whether the
mask came from AI, drawn shapes or a parametric selection. Inspect each affected
skin/bright-material surface from one end to the other, including its interior,
both contours, joints and shadowed portions at native scale, alongside the
parent and actual mask. Coverage regions are bounds around those whole surfaces;
fixed-size crops centered on measurement samples cannot replace them. For
multiple local modules, associate each surface with the relevant runtime mask
page; the first page does not stand for the others. Look for dark islands,
contour bites, abrupt bands and halos while retaining real shading. Review each
changed surface once and reuse that evidence while its mask and upstream
processing remain unchanged.

### Fast coverage checks

Use `scripts/audit_mask.py` on a grayscale AI alpha for inexpensive preflight,
or on an exported runtime mask for actual blend coverage. It locates enclosed
holes and small contour notches, and writes normalized cue regions for the
review-board helper. These are candidates to inspect, not automatic defects:
real gaps also produce cues, and a wholly omitted part can produce none.

For an important narrow part or a repaired omission, place a small verified
interior region that should receive the edit, and a nearby gap/background
region when spill is in question. The same region JSON format is used here:

```bash
uv run "$PHOTO_XMP_SKILL_DIR/scripts/audit_mask.py" --mask actual-mask.png \
  --regions analysis/coverage-regions.json --include intended_interior \
  --exclude nearby_gap --output analysis/mask-check.json
```

The default assertion requires 98% of included pixels to have weight >=0.9 and
excluded pixels <=0.1. Apply it only to verified opaque interiors expected to
receive full weight; adjust the expectation for deliberate partial opacity.
Do not apply this assertion to feathered edges, translucent material or tonal
selections. Assertions test the declared regions, not the entire semantic
object. Topology runs at a recorded analysis size; small-defect diagnosis can
raise `--max-edge`. No-cue output is not a visual acceptance certificate.

Route cues by whole edited surfaces instead of taking the largest few across
the photograph: large real background gaps can otherwise hide a small skin
defect. A coverage-review region surrounds the complete surface and its edge,
unlike the small opaque-interior regions used for assertions. With a runtime
mask, generate the full-size parent/final/mask tiles in one call:

```bash
uv run "$PHOTO_XMP_SKILL_DIR/scripts/audit_mask.py" --mask module-masks.tif \
  --page-name 'exposure • subject light' \
  --regions analysis/surface-regions.json --surface affected_surface \
  --image source=photo.jpg --image candidate=module-masks.tif \
  --review-dir analysis/mask-surfaces --output analysis/mask-check.json
```

Use the actual PageName. The new directory contains overlapping 512-pixel tiles
and their coordinate index; inspect them, retaining parent shading as context.
The grayscale column shows continuous runtime weight. A source without the
local edit can provide geometric context when no full-size parent exists.
`--surface` scopes cue routing, not a semantic pass. If a verified opaque interior
contains a false hole or bite, repair it and freeze an include/exclude assertion
there for the rerun. Move a failed assertion only when the original region is
visibly invalid, not simply to make it pass.

Export the runtime mask during development when substantial silhouette lighting
makes coverage consequential. A full-resolution diagnostic TIFF also contains
the developed image, so use that page for the surface review. Avoid extra mask
exports for color variants with unchanged mask/upstream processing. AI alpha
checks cannot establish actual blend coverage.

### Export the runtime mask when needed

Use darktable's module-mask display or a native TIFF export. This complements
final validation/rendering. On a runtime supporting `--export_masks`:

```bash
mask_check_dir=$(mktemp -d)
darktable-cli photo.jpg subject-r5.xmp module-masks.tif \
  --width 0 --height 0 --hq true --icc-type SRGB \
  --apply-custom-presets false --export_masks true \
  --core --configdir "$mask_check_dir/config" \
  --cachedir "$mask_check_dir/cache" --library :memory: \
  --conf plugins/imageio/format/tiff/bpp=16 \
  --conf plugins/imageio/format/tiff/compress=1 --disable-opencl
magick identify module-masks.tif
```

Inspect the TIFF pages before extraction. In a verified single-masked-module
export, page 0 is the developed RGB image and page 1 is its blend mask. With
multiple masked modules, identify the intended module's page rather than assuming
`[1]`. The audit helper can read TIFF directly with an exact `--page-name` from
its TIFF PageName tags. To extract a confirmed page without thresholding:

```bash
magick 'module-masks.tif[1]' -channel R -separate +channel \
  -depth 16 actual-mask.png
```

ImageMagick here only extracts an already rendered mask; segmentation, path
rasterization, guided refinement, and the local edit remain in darktable.

## Diagnose residual edges before changing strategy

Repair semantic omissions with foreground/background prompts. For an otherwise
correct selection with a few false holes or contour bites, remove only the
verified false subtractive paths or repair/add a fitted path in the same mask
group. Union the coverage before applying the edit: a separate overlapping
exposure module can double the lift. Preserve real gaps; never fill every hole
or expand the entire silhouette blindly. Guided refinement smooths a mostly
correct boundary; it does not recover omitted surfaces. Recheck the repaired
interior and neighboring contour at the original adjustment strength.

If a colored rim persists, compare it with the parent: an exposure lift can
amplify captured fringing or mixed foreground/background pixels. Mask refinement
changes adjustment weights; it does not decontaminate edge color, reconstruct
transparent hair, or recover clipped channels. Do not keep increasing the radius
as a substitute for diagnosing those pixels.

For open-ended lighting, an unstable silhouette may justify a simpler local path,
a broad light field, reduced strength, or upstream tonal work. For a fixed mask
experiment, preserve the prescription and deliver the comparison with specific
remaining defects; successful rendering does not establish naturalness.
