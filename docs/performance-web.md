# Raygun web performance investigation — 0.2.1

## Large-RAW iteration — 0.2.2 (in validation)

Fixture: CC0 Sony ILCE-7M3 compressed ARW, 6000×4000 sensor output,
SHA256 `250784580ea527442c09004417bb0eead484f2bf3ee8f9121a776ac65bb50d0f`.
Chrome headless, 2560×1600, DPR 1, fresh OPFS contexts, two preview workers,
eight Exposure steps and a full-size JPEG90 download. Reports and harness are
under gitignored `target/release-review`. Readiness requires actual photo
selection and completed worker jobs, rather than accepting a catalog count alone.

Targets: export command accepted within 100 ms; preserve editing and cancellation
during encoding; full export time within 20% of the paired baseline; identical
final decoded pixels and metadata; steady draft p95 <=60 ms. Cold first-draft
latency is reported separately. No compiler runs during the final measurements.

Earlier large-RAW preview measurements were discarded: an asynchronous browser
wait did not reliably await the predicate, leaving render jobs overlapping.
The replacement harness explicitly polls completion from Node and requires each
new preview key and idle queues. A downloaded file alone is insufficient evidence.

Fresh paired runs with the 1024-pixel draft experiment measured baseline/candidate
export times of 33.60/41.21 s and 26.54/26.53 s. Candidate command acceptance was
11.3/16.2 ms; baseline acceptance blocked until encoding completed. All four JPEGs
were byte-identical (SHA256
`312d319dc335f5fe6075a91f5889f827cec4d41c5bbf11d0f6ca20ea3eee06e4`).
The substantial timing variation prevents a total-export speedup claim. Steady
draft latency still missed 60 ms. Reports: `*-paired-final*-large-raw.json`.

The final beta caps whole-frame browser drag textures at 512 pixels and restores
normal size on release. Background progress paints run four times per second;
unmodified export pixels are borrowed instead of cloning a 96 MB 24MP buffer.

Three interleaved pairs (candidate/baseline, baseline/candidate, candidate/baseline)
on 2026-10-14, with no compiler active, measured:

| Metric | Released 0.2.1 | 0.2.2 beta |
|---|---:|---:|
| Median steady draft completion | 199.8 ms | 66.9 ms |
| Steady draft completion p95 | 233.4 ms | 83.3 ms |
| Median first draft completion | 449.5 ms | 100.0 ms |
| Export command acceptance | 23.21–24.67 s | 12.3–24.9 ms |
| Median full-size JPEG download | 23.61 s | 25.15 s |

Completion includes the new preview key and idle render queues, so these are
conservative command-to-completion measurements, not measured screen-paint times.
All six runs completed 20 remote jobs with zero inline jobs, browser exceptions or
save errors. Drafts were 511×341; release restored the 1983×1322 preview. All six
6000×4000 JPEGs were byte-identical, including metadata, with the SHA256 above.
Raw evidence: `*-512-{a,b,c}-large-raw.json` and `summary-512.json` under
`target/release-review/`.

Decision: retain the bounded draft and worker export changes for the beta. Export
acceptance and the 20% total-time guardrail passed; median total time increased
6.5%, so this is a responsiveness improvement, not an export-throughput speedup.
The original 60 ms large-RAW p95 goal remains unmet. The browser uses CPU photo
processing (`crates/gpu` has a WASM CPU fallback); async WebGPU integration and
frame/command scheduling need separate profiling. Do not claim full large-RAW
smoothness, mobile performance or completed RapidRAW parity. Deployment is pending.

Browser regression: set `PLAYWRIGHT_MODULE` to an installed Playwright module and
optionally `CHROME_PATH`, then run
`node packaging/web/test-export-worker.mjs http://127.0.0.1:8088`.
This tests the actual WASM, importer, worker and encoder with a generated PNG,
including session destruction; a download alone did not detect the former WASM
native-clock panic. Memory/OPFS/IndexedDB cancellation, worker startup failure,
completion, retry, Original-plus-XMP downloads and stored-original exports after
reload passed on the final beta. Real downloaded pixels distinguish modes 0, 3
and 4; the selected tone mapper and resulting pixels survive persistent reloads.

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
