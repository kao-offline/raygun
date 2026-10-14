<p align="center">
  <img alt="Raygun" src="assets/raygun.svg" width="180">
</p>

<h1 align="center">Raygun</h1>

<h3 align="center">Your photos. Your pixels. Your machine. Now with zap.</h3>

<p align="center">
  <b>Fast photo library + non-destructive RAW developer in pure Rust.</b><br>
  LightCraft UI adapted for Raygun. RapidRAW features land as modes of existing tools. Desktop first, WASM-compatible.
</p>

<p align="center">
  <img alt="Pure Rust" src="https://img.shields.io/badge/pure-Rust-f2a516?style=flat-square&logo=rust&logoColor=white">
  <img alt="egui native UI" src="https://img.shields.io/badge/UI-egui_native-8a5800?style=flat-square">
  <img alt="Desktop Windows macOS Linux" src="https://img.shields.io/badge/desktop-Windows_%C2%B7_macOS_%C2%B7_Linux-8a5800?style=flat-square">
  <img alt="WASM compatible" src="https://img.shields.io/badge/WASM-compatible-8a5800?style=flat-square">
  <img alt="License: AGPL-3.0-or-later" src="https://img.shields.io/badge/license-AGPL--3.0--or--later-blue?style=flat-square">
  <a href="ROADMAP.md"><img alt="Status: Phases 1-4 landed" src="https://img.shields.io/badge/status-Phases_1--4_landed-f2a516?style=flat-square"></a>
</p>

<p align="center">
  <img alt="Raygun icon" src="assets/raygun.svg" width="300">
</p>

## What this is

Raygun builds on LightCraft and uses RapidRAW as a feature reference:

- **LightCraft** (`https://github.com/storytold/lightcraft`) — pure-Rust library, develop pipeline, GPU `wgpu` CPU-exact rendering, command registry, MCP/CLI/control channel. Its `crates/ui-egui` supplies the adapted UI.
- **RapidRAW** (`https://github.com/CyberTimon/RapidRAW`) — selected tone-science and RAW decoding algorithms adapted into existing Raygun tools under the specific licence reviews in `AGENTS.md`. The full engine port is incomplete; see [`docs/rapidraw-port.md`](docs/rapidraw-port.md).

License: **AGPL-3.0-or-later** (see `LICENSE`). Upstream MIT/Apache notices retained in `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`.

Implemented Raygun additions include RapidRAW `.rrdata` adjustment import,
AgX-style/ACES-style tone shoulders, depth-range masks (with a focus fallback
when no depth matte exists), duplicate groups and export-size estimation.
The complete RapidRAW decoder and AI stack have not been ported; predictive Olympus decoding is the first reviewed Rawler adaptation.

Web app: [raygun.kaooffline.top](https://raygun.kaooffline.top).
The existing Cloudflare deployment is documented in [hosting/README.md](hosting/README.md).

## Quick start

```sh
cargo run --release -p lightcraft -- --memory   # throwaway demo session
cargo run --release -p lightcraft -- ~/Pictures/trip
cargo run --release -p lightcraft -- --control 7980
cargo xtask ci                                  # fmt, clippy, tests, layers, assets, wasm
```

Rust 1.90+ required; CI uses stable Rust.

New libraries start empty. First launch imports your photos: File ▸ Import
Photos…, or pass folders on the command line. `--seed-demo` fills a fresh
library with procedural demo photos instead.

On Windows, use Help ▸ Check for Updates… (plus a weekly
background check), then Download Update and Restart and Install Update. The MSI upgrades in place —
existing library files are kept. Administrator approval may be required. macOS/Linux currently download only. Upstream LightCraft work is merged in
by maintainers, never installed over you: see [`docs/upstream-sync.md`](docs/upstream-sync.md).

## Project boards

- Roadmap: [`ROADMAP.md`](ROADMAP.md) — phases, Top Gaps, WASM rules.
- Parity tracker: [`docs/parity.md`](docs/parity.md) — one row per Lightroom feature/menu/shortcut.
- Agent loop: [`tools/agent-loop/`](tools/agent-loop/) — `loop.ps1` runs check → fix → review passes until green.

## Repo map

`geom`, `color`, `raster`, `tiff` → `raw`, `codecs`, `meta`, `develop` → `pipeline` → `catalog` → `engine` → `ui-egui`, `mcp` → apps `lightcraft` (internal compatibility name), `lightcraft-cli`, `lightcraft-web`.

## Branding

- Vector mark: [`assets/raygun.svg`](assets/raygun.svg) (user-supplied original, AGPL).
- White-frame PNG app icon (1024/512/256/…): tracked follow-up — export from the SVG via `packaging/icons.sh` + `cargo xtask ico`, then register under `assets/app-icon/`.
- No Adobe assets ever. Every asset needs an `assets/ATTRIBUTION.md` row in the same commit (`cargo xtask assets` enforces it).
