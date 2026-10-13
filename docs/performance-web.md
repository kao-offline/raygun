# Raygun web performance investigation — 0.2.1

## Contract

Goal: responsive startup and photo editing in the existing Rust/egui WASM app.
Measure first canvas paint, slider command-to-texture time, worker job time,
long tasks, and transferred startup bytes. Use the checksum-verified 0.2.0
release as the baseline, then the candidate under the same browser, viewport,
cache state, fixture, and worker count. Record three runs and raw reports.
Use `?bench&store=memory&workers=2` for the procedural editing fixture, and a
fresh persistent library for normal startup. Keep procedural images opt-in.

Targets: draft worker p95 <= 16 ms and draft command-to-texture p95 <= 60 ms
on the benchmark fixture; remove optional-font waiting from first paint.
Preserve rendering, import/export, persistence, and Chinese glyph support.
Run an additional single-worker/slow-resource case as a beta guardrail.
If this device cannot meet the latency target, report the measured gap and
the responsible boundary rather than claiming general smoothness.

## System map and ranked hypotheses

HTML -> streaming WASM compilation -> font download -> storage/library boot
-> egui/WebGL canvas. Up to four module workers render previews; JSON jobs
and transferred RGBA results cross the worker boundary. Image affinity and
stage caches already exist. Production runs on Cloudflare Workers Static Assets (alternative Nginx serves gzip siblings); asset names remain
stable and must revalidate across updates.

1. A separate ~16 MB Chinese font gates startup even for English; defer its
   completion while preserving eventual glyph support. Verify the resource
   waterfall and first-paint timing before editing.
2. Stale slider jobs may delay the newest preview; inspect queue/coalescing
   and benchmark worker versus command-to-texture time before changing it.
3. Worker startup and memory duplication may hurt small devices; compare
   worker counts before adjusting defaults.
4. Uncompressed font transfer or cache policy may dominate cold loading;
   inspect actual server responses and compressed siblings.

## Research ledger

Verified 2026-10-11:

- [MDN streaming WASM compilation](https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/JavaScript_interface/compileStreaming_static): compile a streamed module once and share it with workers. Direct fit; this app already implements it, so replacing the compiler path lacks evidence.
- [web.dev off-main-thread architecture](https://web.dev/articles/off-main-thread): workers reduce main-thread contention. Direct fit to CPU photo processing; worker communication and memory costs still require local measurement. Existing worker rendering must be preserved.
- [MDN font loading](https://developer.mozilla.org/en-US/docs/Web/API/CSS_Font_Loading_API): font download and activation can be controlled separately. Partial fit: egui renders its own fonts rather than CSS fonts, so deferred activation must update egui's font definitions.

## Evidence and decision

Measured 2026-10-13 using the checksum-verified release ZIPs (0.2.0 baseline,
0.2.1 candidate at `808436d9d5fa842e4dcc9ab084f6476481a30b36`). Chrome headless,
1280x800, DPR 1, three fresh contexts per case, local HTTP with gzip siblings,
memory storage, procedural benchmark, no concurrent compiler. Percentiles use
nearest-rank over the 24 slider samples. First-frame timing is the app's canvas
metric; it is not the loading text's browser FCP.

| Metric | 0.2.0, two workers | 0.2.1, two workers | 0.2.1, one worker |
|---|---:|---:|---:|
| Median first canvas frame | 440.6 ms | 225.3 ms | 220.9 ms |
| Draft command-to-texture p95 | 34.0 ms | 34.0 ms | 34.0 ms |
| Draft worker job p95 | 10.6 ms | 11.2 ms | 10.9 ms |
| Median benchmark thumbnail completion | 20.57 s | 20.84 s | 26.63 s |

A separate five-second font-delay case (three fresh contexts, empty library)
reduced median editor-ready time from 5227.9 ms to 213.3 ms. The eventual Chinese
font still loaded: Simplified Chinese menus and an imported Chinese filename
were inspected. The normal first-frame median improved 49%; editing latency
was preserved, not improved. Thumbnail completion was within 2% of baseline;
its expensive procedural fixture remains a separate bottleneck.

Real browser smoke testing passed on the candidate and production host: fresh
OPFS library at zero photos, actual PNG drop/import, JPEG export before/after
exposure +1 (sample RGB 88/82/80 -> 123/113/111), photo/settings persistence after
reload, no browser exceptions/save errors. `?bench&store=opfs&workers=999` used
memory with at most four workers and left persistent edits unchanged.

Raw reports and screenshots are local recovery evidence under
`target/release-review/`: `baseline-bench.json`, `candidate-bench.json`,
`candidate-one-worker-bench.json`, both `*-slow-font.json`,
`performance-summary.json`, and `live-021-smoke.json`. No media is committed.
The production HTML/JS/WASM/worker/font/icon hashes match the release bundle;
stable URLs respond with `Cache-Control: no-cache`.

Production beta on `raygun.kaooffline.top` used the same browser/viewport/fixture
and two workers, three fresh contexts: median first canvas frame 1170.7 ms,
draft preview p95 34 ms, worker p95 11.5 ms, thumbnail completion median 20.38 s,
no exceptions. Network loading makes these startup numbers different from the
local comparison; no production-before/after speedup is inferred. Raw evidence:
`target/release-review/live-bench.json`.

## thinkBETTER iteration 1

Goal: draft worker p95 <=16 ms, command-to-texture p95 <=60 ms, no optional-font
startup gate, preserved import/export/persistence/glyphs.
Baseline: 440.6 ms first frame, 34 ms preview p95, 10.6 ms worker p95.
Current best: 225.3 ms first frame, 34 ms preview p95, 11.2 ms worker p95.
Primary delta: defer optional font download/activation until the editor starts.
Guardrails: three-run single-worker and delayed-font cases; real OPFS smoke flow.
Evidence: real release ZIPs, reports above, `cargo xtask ci`, packaging regressions.
Research update: existing streaming-WASM and worker mechanisms were preserved;
measurements do not justify replacing the renderer or changing worker defaults.
Decision: COMPLETE for this bounded startup/preview experiment.
Why: measured targets and correctness guards passed. This small procedural loupe
does not establish smoothness for large RAWs, mobile devices or large libraries.
Next hypothesis: profile a representative large-RAW browser workload before
changing decode/thumbnail scheduling; exports still run on the main thread.
