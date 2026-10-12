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

Baseline measurements and candidate results are pending. No performance
improvement is claimed from code inspection alone. Validation is compiling
concurrently; repeat measurements under equal contention or after it ends.

Decision: CONTINUE — capture baseline, then test the highest-impact verified
hypothesis with a reversible change. No framework or renderer migration.
