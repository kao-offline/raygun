# Portrait skin rendering

Use this reference when skin is a material part of the edit. Select the portrait
profile from the image, light, and requested result rather than from one keyword.
For an uncertain sweet-portrait endpoint or a documented audit, use
[sweet-portrait-self-calibration.md](sweet-portrait-self-calibration.md) for
targeted comparisons and optional evidence records. Routine acceptance is visual.

## Choose the profile

### General portrait

Preserve a believable complexion under the intended light and style. Skin may
be warm, cool, dark, muted, dramatic, or low-key when that serves the photograph.
Keep facial dimension, texture, highlight detail, and plausible color variation.
There is no universal hue, luminance, saturation, or histogram target.

### Sweet portrait / 糖水人像

Use when the user explicitly asks for 糖水人像 or when the source and overall
brief point toward a bright, clean, soft, refined portrait with luminous
白皙通透/白里透红 skin. Relevant signals can include a person-led composition,
open or intentionally lifted light, airy/clean/romantic color, soft rendering,
and an emphasis on polished skin. Judge the combination rather than requiring a
specific phrase or treating any single adjective as decisive. Its target has
four independently controlled parts:

- **Clean fair base:** ordinary forehead, jaw, arms, and legs are naturally
  fair and free of diffuse yellow/orange/red or cool-gray/cyan/green
  contamination.
- **Luminous skin:** ordinary facial planes read bright and clear at the
  intended viewing size, with highlight headroom and gentle modeling. Use the
  skin histogram when placement is uncertain, not to set a universal endpoint.
- **Localized vitality:** lips, cheeks, ears, fingertips, and joints remain a
  little warmer/redder than adjacent ordinary skin.
- **Subject hierarchy:** the person reads clearly without forcing black hair,
  dark clothing, or the entire scene to the right.

## Diagnose by region and light

Inspect ordinary face skin and materially visible hands or limbs together in
context. Use the sampling and distribution procedures below only when color or
lightness is uncertain, a correction needs diagnosis, or an audit requires them.
Do not create a skin analysis package solely because a person is in the frame.

When measuring, sample face skin and important hands or limbs separately.
Avoid hair, makeup, lips, specular highlights, deep occlusion, and background
edges in the ordinary-skin sample. Add separate lips/cheeks regions when judging
vitality or changing red/orange. Distinguish direct light, midtones, open shade,
and deep shadow; a single average can hide red highlights and dark/cyan shadows.

For a sweet-portrait lightness question, inspect ordinary-skin distribution
shape, P25/P50/P75, the proportion above the display midpoint, and channel clipping. A
thin bright tail from a specular patch does not establish luminous ordinary
skin. Placement depends on complexion, makeup, illumination and the brief. Use
the full-frame histogram for scene safety and the whole-subject histogram only
for hierarchy.

If ordinary skin still reads dim in the completed composition and useful
headroom remains, continue its luminosity work. Back off when the skin itself
loses texture, useful color or facial planes. Compare important limbs
individually so a bright face does not hide muddy knees, arms or hands. Also
compare ordinary face/neck and limb color in one figure-context view: individual
regions can each look plausible while their relative redness or warmth feels
inconsistent. Use the same-region before/after changes to distinguish a captured
light difference from one introduced by selective cleanup or lifting. A captured
difference can still need harmonizing for the requested finish. Diagnose the
remaining difference after the first lift, then correct the region that is
visibly wrong:

- **Healthy face, gray/cyan limbs:** hold the face and test a local limb-color
  correction. Brightening alone can leave gray skin brighter; desaturating the
  good face creates another defect. Restore only missing warm/red variation
  with a gentle color balance or hue correction, keeping shadow depth, joint
  color and neutral socks/clothing.
- **Diffuse red/orange face, healthy limbs:** clean the affected facial base
  while preserving lips/cheeks and the good limbs. Narrow the hue/spatial reach
  if a broad desaturation makes the face lifeless.

Compare both in an enlarged full-figure view. A cast-dependent warm/magenta
correction can suit shaded skin even in a cool setting; it is not a general
skin tint. Refine strength or reach when a trial is excessive. An unsuccessful
face-desaturation trial does not establish that gray limbs must remain. Do not
darken a good face or erase all skin color to make the averages match.

## Quantify fair skin as a joint distribution

Brightness alone cannot distinguish fair, refined skin from brighter orange
skin. When that distinction needs measurement, use
`analyze_photo.py --skin-region NAME` on the same visually verified pixels in
the source, accepted parent, candidates,
and—when available—a user-approved anchor. Inspect the generated region map
before accepting any measurement. Invalid samples that include hair, eyes,
lips, makeup, uniform, background, or specular points must be moved or removed.

Read the four CIELAB distributions together:

- **L*** measures perceptual lightness and complements the display-luma
  histogram. Use it to locate which ordinary-skin planes need opening while
  retaining texture and highlight headroom.
- **a*** measures green-to-red. Preserve plausible ordinary-skin warmth, but do
  not raise it across the whole face merely to create cheek vitality.
- **b*** measures blue-to-yellow. When diffuse orange/yellow is the problem, a
  downward or controlled b* movement is part of cleanup.
- **C*** measures total chroma. It reveals broad color contamination even when
  the hue still looks superficially skin-like.

Do not collapse these axes into a black-box “fairness score” or impose one
absolute target on different complexions and illuminants. Compare distribution
shape and P25/P50/P75 against the parent and a relevant approved anchor. A
successful fair-refined direction commonly combines any needed L* increase
with controlled or reduced excess b*/C*, while ordinary-skin a* moves
only as much as the light and complexion justify. If a*/b*/C* all rise broadly,
the result is becoming more red-orange and concentrated, not more white and
clean—even when exposure has increased.
Conversely, if a cool environment grade causes a broad decrease in a* and/or b*
across ordinary face skin and several visible limbs, the skin is shifting toward
green and/or blue. Judge the size and motivation visually: consistent diffuse
movement is different from a plausible local edge reflection.

In a scene-adaptive person-first sweet portrait, allow the environment to carry
more deliberate color than central skin. This makes the skin-parent regression
especially important in foliage, warm backlight, ambient night, neon, and mixed
flash. Keep small motivated rim or edge reflections, but reject broad
contamination across the ordinary face and visible limbs: foliage must not make
skin cyan/green, and lamps or autumn color must not turn all ordinary skin
orange/red. Correct base skin independently from the environment instead of
globally neutralizing the scene. Read
[scene-adaptive-sweet-portrait.md](scene-adaptive-sweet-portrait.md).

Inspect shadow, midtone, and highlight bands separately. This catches the common
failure where highlights look pale but shaded skin remains dark orange, or where
an average hides yellow highlights. Treat lips and localized cheeks as separate
vitality diagnostics; their color should remain distinct from the cleaned
ordinary-skin base rather than being used to justify whole-face red/orange.
After a broad ordinary-skin cleanup, recheck these accents explicitly: a clean
face with beige lips or gray cheeks is not a successful person-first finish.

## Keep skin decisions attributable

Start from a credible tonal parent (global and local structure as needed) and
use only the axes the photograph needs:

1. **Base color:** correct a genuine illuminant error or diffuse ordinary-skin
   cast before changing skin brightness. Use Lab distributions if needed to
   diagnose cleanup without treating lips/cheeks as evidence that all skin is too red. Target
   excess yellow/orange where it exists; whole-skin desaturation is not the
   default cleanup when healthy reds already work.
2. **Luminosity:** compare protected lower-mid/midtone lifts when ordinary skin
   needs opening. Preserve its color and modelled planes; use the tonal guide's
   control-response comparison if brightening skin also washes dark materials.
3. **Vitality:** restore or restrain warm-red accents without spreading redness
   across ordinary skin. Fairness and vitality are different jobs: do not use
   orange saturation as a substitute for healthy lips/cheeks, and do not whiten
   those accents until they turn beige or gray.
4. **Separation:** if the skin is right but the figure still disappears, adjust
   whole-subject, clothing, hair, or background relationships without reopening
   the accepted skin target.
5. **Environment grade and regression:** explore the scene's color story while
   treating the accepted skin render as locked. Keep motivated local reflection,
   but protect or counter-correct diffuse spill instead of weakening an unrelated
   skin or subject-light axis. Compare retained renders; use the optional audit
   guide if that regression remains unclear.

These are separable decisions, not five compulsory rounds. Keep useful parents
when changing a material axis, and bracket its target if the endpoint is uncertain.
A tidy ladder of insufficient settings is not complete. In shaded skin, lift protected
lower midtones rather than adding global warmth, which turns darkness muddy
brick-red.

Compare ordinary skin, lip/cheek context and affected setting colors before and
after cleanup, then skin/materials before and after grading, using the actual
retained renders. These comparisons do not require separate stage exports. A muted skin
parent can pass every later regression while preserving the wrong endpoint.
If reduced desaturation restores lips but also returns yellow skin, separate
the controls: retain red vitality while cleaning the affected yellow/orange.
Likewise, if skin cleanup drains attractive warmth from wood, architecture,
foliage or sunset light, narrow that cleanup to skin instead of rebuilding the
setting's color afterward. Choose hue-family shaping or spatial protection from
the overlap in this image; a global correction is suitable when its full-frame
effect serves both skin and setting.

When a revision swings from too gray to too red, compare the accepted state
with the first color-changing operation. A global warmth/channel move followed
by strong red/orange desaturation can leave pale highlights, gray limbs and
uneven warm patches. Correct the original reach or color response, then restore
only the needed local variation. Evaluate face, neck, hands and legs together
after any broad warmth or red/orange change; a brighter cheek is not evidence
that ordinary skin elsewhere stayed healthy.

## Controls and masks

Use White Balance for a genuine scene illuminant error and Color Calibration
when the known illuminant/chromatic adaptation or camera-channel response is
the actual cause. Use Color Equalizer for smooth hue-family correction, Color
Balance RGB for the broad palette, and a masked luminance control for local
light. White Balance and Color Calibration are technical foundations, not
shortcuts for making skin fairer or applying a look. These controls are not
semantic: hue or tonal overlap can couple skin and setting. Use the stage
comparison above to choose their reach. A master look-intensity parameter may scale a coherent
environment story, but it must not also scale skin-base cleanup, skin luminosity,
vitality, or subject separation when those jobs need independent endpoints.

Measurement regions and editing masks are different objects. Excluding lips
from a skin statistic does not protect them from a broad face ellipse or a
hue-range mask. Inspect the actual rendered effect on lips, cheeks, ears and
limbs. Where cleanup would erase vitality, narrow its color/spatial reach or
compensate only the affected accents. A naturally subdued lip may need no added
color; a lively close-up must still read alive at intended viewing size.

For broad subject underexposure or a flat/gray result, use
[tonal-hierarchy.md](tonal-hierarchy.md) to select curves, contrast/fulcrum, zone
work, or local light from the diagnosed problem. Skin color acceptance and
subject brightness remain separate axes.

## Batch consistency and acceptance

Compare representative direct-light, soft-light, shade/backlight, and—when the
set includes them—ambient-night and flash/mixed frames side by side. Transfer
relationships—not numeric settings—from a successful frame: skin versus textured
white, vitality versus ordinary skin, face versus limbs, and subject versus
background. A bright easy frame cannot prove an underlit wide or night frame is
finished.

For general portraits, accept skin when it matches the requested rendering and
remains healthy, dimensional, textured, and coherent with the light.

For sweet portraits, the four profile targets must hold together in the
completed composition and across visible limbs. Follow the main skill's visual
and export checks; use the acceptance reference only for unresolved endpoint
comparisons or an optional strict audit.
