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
| Saved edits | New `.rrdata` tone-mapper imports select modes 3/4; old modes 0/1/2 remain unchanged | Calibration, masks, geometry and external LUT mapping remain incomplete |
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
