# Technical foundation and output detail

Read this only when the capture or output-detail gate needs an edit. These tools
remove a diagnosed limitation; they are not a checklist and should not quietly
become the creative look.

## Freeze geometry before spatial evidence

Resolve EXIF orientation first, then composition crop, horizon/verticals, and
perspective. Judge the crop at delivery size as well as full frame: preserve
intentional headroom, limb joints, gaze space, and useful environmental context.
Apply only the perspective correction the picture needs; forced architectural
perfection can distort people and natural scenes.

Normalized semantic regions and drawn/AI mask prompts belong to one geometry. If
crop or perspective changes, do not reuse their coordinates by habit. Render the
new geometry, redraw them, and restart aligned comparisons from that parent.

## Establish technical color

White Balance establishes the capture illuminant. Color Calibration handles
chromatic adaptation and camera/channel response. Prefer the exact image's
as-shot, camera, or auto-derived state, especially for RAW; inspect plausible
neutrals, skin, whites, shadows, and motivated colored light together.

Use White Balance when the whole scene has an illuminant error. Use Color
Calibration when the illuminant model, adaptation, or channel relationship is
the problem. Mixed light may require a locally protected compromise rather than
one globally neutral answer. Lock a credible technical-color parent before the
creative palette. Do not use technical color merely to whiten skin or spray a
cool/warm style over the frame.

When explicit coefficients are not known, inherit the same photograph's module
and make relative warmth/tint changes, bracketing when the endpoint is uncertain.
Inspect skin/material color and neutral anchors after a material change; add Lab
measurements if visual judgment is inconclusive. Reopen later color and skin decisions
because their earlier measurements are no longer comparable.

## Denoise only visible noise

Inspect representative shadows, skin, flat color, and fine texture at 100%.
Distinguish luminance grain, chroma blotches, demosaic artifacts, compression,
and intentionally retained texture. Prefer the same-image profiled-denoise state
or the runtime's camera-aware profile; never transplant a noise model from a
different photograph.

Bracket the minimum strength that removes distracting noise at the intended
output size. Reject waxy skin, smeared hair/fabric, chroma bleeding, or false
edge smoothness. Denoise before Diffuse or Sharpen, then reevaluate the latter; a
sharpening decision made on noisy pixels is no longer valid.

## Separate haze from flat tone

Use Haze Removal when distant contrast/color progressively collapses through air
or when a deliberate atmospheric effect is requested. Compare near and far
materials, sky transitions, silhouettes, and color gradients. Use restrained
strength and stop before halos, dirty shadows, clipped color, or unnatural depth
compression.

A uniformly gray image with weak black/white anchors is usually a curve, levels,
contrast/fulcrum, or tonal-zone problem. Fixing it with dehaze tends to create
hard edges and distorted color without restoring the intended hierarchy.

## Use channel curves for a channel-level job

Use a linked, color-preserving RGB curve for luminance shape. Use independent
R/G/B curves when the intended result depends on channel-specific contrast, a deliberate
shadow/highlight color split, or a residual channel bias that safer color tools
cannot express. Keep x nodes monotonic, bracket modest moves, and inspect neutral
anchors, skin, gradients, and channel clipping. If the need is simply family
color or tonal grading, prefer Color Equalizer or Color Balance RGB.

## Finish texture at the viewing scale

Diffuse or Sharpen is scale-specific. Name the job before selecting a preset or
coefficients: deblur, capture sharpening, local contrast, diffusion/bloom, or
output sharpening. Prefer a matching active-runtime preset as a known baseline,
then tune one coherent behavior. It does not repair missed focus, a bad mask,
flat global tone, or over-denoised detail.

Inspect critical edges, pores, hair, fabric, foliage, and smooth gradients at
100%, then judge the whole photograph at delivery size. Reject halos, ringing,
crunchy texture, plastic skin, false microcontrast, or diffusion that erases the
subject hierarchy. Recheck after resizing or changing export dimensions.
