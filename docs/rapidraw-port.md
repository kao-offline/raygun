# RapidRAW adaptation

Raygun keeps its pure-Rust engine and browser-compatible architecture. It does
not embed RapidRAW's Tauri application or native runtime. A full engine port is
not complete: selected algorithms are adapted individually and verified against
the pinned sources and independent fixtures.

Sources reviewed: RapidRAW `74597d066685283588502edbecd747bfe80404cf`
(AGPL-3.0), RapidRAW-DngLab `a32bc1ff1f4f3c56d4d3226082f6eb17414f9159`
(Rawler 0.7.1, LGPL-2.1). See `NOTICE`, `LICENSE-LGPL-2.1` and the review
in `AGENTS.md`. No camera matrices, model weights or LUT assets were imported.

| Area | Implementation | Remaining scope |
|---|---|---|
| AgX | Actual inset/outset matrices, log encoding and sigmoid LUT; Rust CPU and WGSL | Whole-pipeline visual equivalence and HDR validation |
| Simplified ACES | RapidRAW's fitted SDR transform; Rust CPU and WGSL | This is a fitted operator, not the entire ACES colour-management system |
| Saved edits | New `.rrdata` tone-mapper imports select modes 3/4; old modes 0/1/2 remain unchanged | Calibration, masks, external LUT mapping and out-of-frame straighten remain incomplete |
| Olympus predictive ORF | Checked Rust decoder adapted from Rawler; independent sensor-pixel regressions for E-M1 and E-M10 III | Other models, 14-bit and multi-strip coverage |
| Other RAW decoders | Existing Raygun decoders retained | Compressed Samsung, Nikon HE, Canon sRAW and remaining coding variants |
| Lens correction | Existing Raygun manual corrections | Pure-Rust Lensfun database adaptation, attribution and profile coverage |
| AI | Existing opt-in Raygun Rust paths retained | Model-by-model licences, compatible inference and numerical equivalence |
| GPU | Existing Rust/WGSL pipeline extended | RapidRAW guided-filter and remaining shader algorithm parity |
| Browser exports | Disposable module worker, shared encoder, asynchronous original loading, progress and cancellation; real memory/OPFS/IndexedDB completion, failure, retry and reload checks | Large-RAW p95 remains 83 ms against the 60 ms goal; total export throughput is not improved (see `performance-web.md`) |

The native ORT runtime, Lensfun C/C++ library, Tauri and unsafe Rawler container
code are excluded. Excluded native implementations must be replaced by safe
Rust equivalents rather than silently enabled only on desktop.

The new tone modes are selectable in Edit → Light → Tone Mapper. Modes 1 and 2
are explicitly labelled legacy approximations so existing edits do not change
when the new transforms land. New modes use SDR output limits; they do not
establish HDR parity.

## Import saved RapidRAW edits

Select the target photos, then choose **File > Import RapidRAW Adjustments...**.
Paste the JSON contents of a `.rrdata` sidecar and confirm. Both a flat adjustment
object and a wrapper with an `adjustments` object are accepted (maximum 1 MiB).
The selection is captured when the dialog opens; the batch is one undo step.
Invalid JSON keeps the dialog open. Unsupported fields appear in the result toast
and in the command response, including per-photo results for batches.

Pixel crops are rounded and clipped to each photo's EXIF-oriented dimensions after
`orientationSteps` (0..3 clockwise quarter turns). Explicit percent crops are also
accepted. Horizontal/vertical flips and clockwise straighten (-45..45 degrees)
are converted to Raygun's crop order without changing their framing. Geometry is
applied atomically per photo; malformed geometry and rotations whose crop extends
outside the source are reported as unmapped, while supported tone edits can still
import. This avoids silently substituting Raygun's clamped border for RapidRAW's
transparent border. `crop: null` resets the rectangle; aspect-ratio UI locks remain
unmapped. Existing Raygun lens/perspective edits remain in effect.

Geometry conversion is original Rust code referencing the saved-edit schema and
operation order at the pinned RapidRAW revision above; it imports no new runtime,
assets or dependencies. This is edit interchange, not whole-pipeline pixel parity.
