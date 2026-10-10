# Raygun GitHub project setup

Create a GitHub Project (Boards) named **Raygun** with these columns, then link this repo:

- `Phase 0 — Fork green` (done: UI reused verbatim, `cargo check -p lightcraft` green, `cargo xtask assets` 43 files attributed)
- `Phase 1 — Sidecar/import compat` (`.rrdata` + preset/LUT importer → develop controls, CLI-verified)
- `Phase 2 — Pipeline modes` (AgX/ACES, exposure set, Kelvin WB, perceptual HSL, guided filter; CPU+GPU+equivalence test per block)
- `Phase 3 — Masks + Detail AI` (classical fallback first, optional ONNX subject/sky/depth, NIND/BM3D, LaMa retouch; AI-free toggle)
- `Phase 4 — Library/culling + export` (masonry, 6-up cull, auto-cull, HDR deghost, batch export modes)
- `Phase 5 — WASM check` (`cargo xtask web --serve`, OPFS smoke, export-download smoke)

Labels: `ui-reuse`, `pipeline`, `masks`, `detail`, `library`, `export`, `wasm`, `license`, `assets`, `loop`.

Banners live in `README.md` (shields: Pure Rust, egui native, Desktop, WASM-compatible, AGPL-3.0-or-later, Phase 0 fork).
Icon source: `assets/raygun.svg`. White-frame PNG export is tracked work: render via `packaging/icons.sh` + `cargo xtask ico` into `assets/app-icon/`, then update `ATTRIBUTION.md` in the same commit.

Milestone rule: one task id per commit, only green states (`tools/agent-loop/loop.ps1` + `review.ps1`).
