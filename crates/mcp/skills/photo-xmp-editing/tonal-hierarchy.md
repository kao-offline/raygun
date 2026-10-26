# Tonal structure and subject hierarchy

Use this reference when a result looks gray, flat, muddy, dim, loses bright-area
gradation, or lacks subject separation. Read structure before choosing a control.
Soft or low-saturation styling does not imply compressed luminance.

## Set the scene lightness before refining the person

For an airy daylight brief, identify the surfaces carrying openness: luminous
walls, pale architecture, bright foliage or a clear sky, as the photograph
provides. A shaded subject can belong in a bright space. Preserve or develop
that space while shaping the person; portrait emphasis can come from skin,
color, focus, framing and dark clothing as well as a darker background.

Distinguish a **supporting bright area** from an **attention competitor** by
its effect on the full composition. Brightness or large area alone does not
make a surface distracting. If the scene looks dull despite adequate facial
lightness, compare a more open midtone structure with black anchors retained.
If one bright floor, stripe or window pulls attention, restrain that area or
tonal zone while keeping useful surrounding light.

## Place bright materials and sky with the subject

In direct sun, backlight or a high-dynamic-range scene, establish the bright-area
rendering alongside subject and scene midtones. Inspect the brightest diffuse
white, sky and actual glints separately. A readable face and low clipping count
do not establish a finished tonal relationship. Judge two qualities:

- **Variation:** folds, planes, warm/cool changes and gradients that the source
  can retain. One clipped channel or a small white core does not erase all of
  the surrounding information.
- **Placement:** the brightness, area and visual weight of the light relative
  to the person. Even a textureless area can be placed more gracefully. A source
  limit constrains recovery, not the final brightness or atmosphere.

When a broad bright area lacks shape or overwhelms the intended mood, compare
its current rendering with a meaningfully quieter one while keeping the face
and useful scene midtones similarly open. Judge the whole frame as well as the
material crop. Stop where the light retains its energy and color transition
without making white cloth chalky, the sky empty, or the setting uniformly dull.
For backlight, preserve the progression toward the light source and its warm/cool
atmosphere; pale surroundings alone do not supply depth.

Choose the control by its rendered effect on both lightness and color:

- For a linked RGB curve doing luminance work, start with its color-preserving
  default. `--preserve-colors 0` applies the curve to channels separately: a
  lifting, flattening shoulder can push pale colors toward white. Reserve that
  behavior for a deliberate color choice. Check skin, white fabric and sky
  when changing this mode; it is not merely a contrast switch.
- If the curve crowds bright values together, start earlier in the pipeline:
  use a smooth tone-equalizer upper-zone run to move the light into a useful
  part of the curve, or reshape that part of the curve itself. Check which
  scene-exposure bands actually reach the material; their names and curve
  coordinates are not display-luma values. If an endpoint change barely affects
  the render, test the earlier placement rather than refining that endpoint.
  Rebalance affected useful midtones before judging the complete alternative.
- Lowering a material and compressing its contrast are different operations.
  Keep separation between lit planes, folds and adjacent tones. Compare stage
  renders to find where shape first disappeared, then change that mapping or
  the light entering it. Reducing a later highlight-compression control cannot
  restore contrast already lost upstream. A smaller near-white count does not
  establish better material shape.
- Use local material control when subject and environment overlap tonally. If
  preserving sky color reveals an unwanted cloth or skin cast, correct that
  material through its own color control rather than washing out the scene.

Compare sky away from the light source and its progression toward that source.
If a lower sky becomes uniformly gray, develop its surrounding gradient and
retained warm/cool color before comparing the finish. The brightest small core
may remain white while the surrounding light gains depth. Keep this spatial
falloff consistent with the captured light; lowering a plateau alone neither
recovers texture nor completes the atmospheric choice.

## Read tone as distributions

Compare the source, accepted parent, candidates, and a relevant user-approved
anchor when one exists. The measurements are relationships, not universal
targets:

- **P05, P50, P95:** robust dark, middle, and bright placement.
- **P95−P05:** robust end-to-end span. It reveals broad compression without
  letting a few clipped pixels dominate.
- **P75−P25:** middle-half span. It helps identify weak midtone separation.
- **P50−P05 / P95−P50:** lower and upper spans. They show which side was
  compressed.
- **Standard deviation:** a supporting measure of overall luminance variation;
  composition can influence it, so compare only aligned renders of the same
  photograph.
- **Black and textured-white anchors:** semantic confirmation that the measured
  endpoints still look photographic.

A simultaneous rise in P05, fall in P95, and reduction in both robust span and
standard deviation is strong evidence of inward tonal compression when the
render also looks veiled. A narrower span can still be intentional in fog,
high-key, matte, or muted work; use the brief and visual result to decide. Do
not chase the source or anchor numerically when its clipping or contrast is a
defect.

“Gray” may instead mean insufficient chroma. Check luminance spans and
saturation/chroma separately. Restore tonal shape with tonal controls; adjust
color only when the palette itself lacks the intended color separation. Adding
saturation cannot repair a flat curve.

## Quantify subject placement and separation

Global histograms can hide a dim person in a bright scene. If visual review
leaves subject placement uncertain, define and visually verify:

- a broad `whole_subject` region for hierarchy;
- face, ordinary skin, or important light clothing for material placement;
- one or more nearby environment regions under comparable light, excluding
  sky, unrelated specular highlights, and arbitrarily dark objects.

Use `--histogram-region` for the subject and `--region-pair` for explicit
subject-versus-environment comparisons:

```bash
uv run "$PHOTO_XMP_SKILL_DIR/scripts/analyze_photo.py" \
  --source source/photo.jpg \
  --candidate parent=previews/accepted.jpg \
  --candidate revised=previews/revised.jpg \
  --parent revised=parent --regions analysis/regions.json \
  --histogram-region whole_subject \
  --region-pair subject_vs_environment=whole_subject,nearby_environment \
  --output-dir analysis/tonal-round
```

Read the subject P25/P50/P75 and its distribution shape, then the change in
subject-minus-environment mean/P50. A dark skirt or black hair can pull a broad
subject measurement left, so confirm face/skin, light clothing, and dark-material
detail separately. A low subject P50 is not itself a failure when the lighter
quartile is readable and motivated dark material retains separation. Conversely,
a high positive subject/background difference is not success when it was created
by an obvious local light field. The goal is intentional hierarchy, not a fixed
positive difference or identical placement across lighting classes.

When the intended sweet-portrait finish remains uncertain, use
[sweet-portrait-self-calibration.md](sweet-portrait-self-calibration.md) for a
targeted endpoint comparison. Its detailed audit is optional; a result should
meet the brief as well as improve on its source.

## Match controls to the diagnosed job

1. **Gross placement:** use global exposure only when the whole capture baseline
   is displaced. Keep it modest and protect bright materials/highlights. Do not
   use a large exposure move to solve flat tone or one dark subject.
2. **Black/white structure:** use levels, a monotonic RGB curve, or Color Balance
   RGB contrast and fulcrum to place actual dark materials and midtone slope.
   A curve passing through zero can still raise every visible dark tone too far.
   Select its color behavior with the bright-area guidance above.
3. **Uneven tonal zones:** use a smooth tone-equalizer run to place affected
   low, middle or high zones independently. Adjust the zones the image needs;
   blanket shadow recovery plus highlight suppression readily creates a veil.
4. **Underlit subject:** after choosing scene lightness, open the skin or other
   underlit material with a response concentrated on its useful tones. A whole
   person mask also includes hair, eyes, dark clothes and bright whites; their
   exposure needs can differ. Use a shaped curve/zone response or material reach
   that preserves those relationships. Choose the spatial boundary through
   [masking-workflow.md](masking-workflow.md).
5. **Color separation:** adjust chroma or the palette after tonal structure is
   credible. Preserve accepted skin; reopen its review if tone changes its
   visible color or shape.

Choose by the rendered response, not the slider name. For example, Basic
Adjustments brightness can substantially lift dark material along with skin.
If it creates a veil, compare exposure or a shaped midtone curve at similar
face lightness; neither the same numeric strength nor a weaker face is a fair
comparison. Inspect whites as well, since exposure can move them farther.
Use the simplest control that produces the intended light and material shape.

Judge dark material in both directions: lifted blacks can look washed gray,
while a steep toe can merge folds, strands and eye/lip shadows into hard black
shapes. Density is not a goal to maximize. Compare placement and internal span
through the stages, including technical to tonal, with a crop spanning actual
folds or strands. For a soft portrait, retain dark-material shape without making
facial features look inked. A few surviving strands do not excuse a gray wash;
a low black percentile does not excuse lost folds.

Locate the first damaging response and compare a gentler one at similar face
lightness. A large global midtone lift paired with a steep black toe often needs
different scene/local allocation, not more contrast. Correct the relevant curve,
zone or material reach while retaining luminous skin and useful scene midtones.
Do not force source blacks back when they were crushed, or compensate for a
local tonal error with global darkening/dehaze or blanket shadow recovery.

## Choose scene and subject allocation

The tonal parent includes global **and local** structure. When a wide daylight
portrait has a shaded person in a light setting, compare two plausible
allocations if the choice is unresolved: more open scene midtones with less
local lift, and a quieter scene with more subject recovery. Bring the faces to
similar intended lightness before judging the surrounding space. Select for
the requested atmosphere, depth, bright materials and natural transitions.
Trying two local strengths on one fixed global curve does not test this choice.

Separate a nearby-environment lighting comparator from an
`attention_competitor` such as sunlit pavement, sky or a saturated background
strip. Their roles differ. Judge a competitor's area, contrast and color at
full-frame size, then restrain only what actually distracts from the intended
subject. Include technical → tonal changes in this review; later stages cannot
reveal an attention loss already accepted into the tonal parent.

Use the main workflow's completed-look selection after allocation. A control
that damages another material before solving its target needs different reach
or tonal placement. Let atmosphere, facial shape and material gradients decide;
the distributions explain those observations.
