# Upstream sync: aggregating LightCraft updates into Raygun

Raygun is a fork of [`storytold/lightcraft`](https://github.com/storytold/lightcraft).
This repo tracks upstream through the `upstream` git remote (read-only; Raygun
never pushes there):

```sh
git remote add upstream https://github.com/storytold/lightcraft.git
git fetch upstream
```

## Merge routine (maintainer)

1. `git fetch upstream` and pick the target (`upstream/main` or a tag like `v0.5.0`).
2. Create a pickup branch off Raygun `main`:
   `git checkout -b upstream-pick-<tag>`.
3. `git merge <target> --no-commit`, then review file by file:
   - **Engine/pipeline/feature code** (`crates/*/src/**`, `apps/*/src/**` logic):
     take upstream, then re-apply the Raygun rebrand map below.
   - **Raygun-owned files** (never take upstream's): everything listed under
     *Raygun-owned* below, plus `crates/engine/src/rrdata.rs`,
     `crates/engine/src/update.rs`, `crates/engine/src/cmd/update.rs`, `crates/ui-egui/src/panels/updates.rs`,
     `xtask/src/icns.rs`, `hosting/`, `docs/upstream-sync.md`.
   - **Locales** (`crates/ui-egui/locales/*.json`): take upstream's new keys,
     then re-run the brand swap (`LightCraft` → `Raygun`) on changed lines only.
     Every `*-formats.json` catalog must keep identical key sets or the build fails.
4. Run the agent loop (`tools/agent-loop/loop.ps1`, then `review.ps1`) and fix.
5. `cargo xtask parity` — upstream feature work lands with its parity rows;
   keep Raygun's added commands working.
6. Commit as `M#: merge upstream <tag>`, push, let CI build, release.

## Rebrand map (re-apply after every merge)

User-visible identity lives in these places; upstream edits nearby must be
re-branded before commit:

- App name/title/about: `crates/ui-egui/src/links.rs` (`APP_NAME`),
  `menus.rs` (Help/File labels), `panels/dialogs.rs` (About),
  `apps/lightcraft/src/native_menu.rs`, `apps/lightcraft/src/main.rs`
  (window title, `APP_ID`), `panels/second.rs`, `panels/topbar.rs`,
  `crates/ui-egui/src/lib.rs` (save notice), `widgets.rs` (preview message),
  `panels/library_problem.rs`, `panels/settings.rs` ("Raygun Default").
- Folders/env: `crates/engine/src/library.rs` (`DEFAULT_NAME`),
  `crates/engine/src/config.rs` (`config_dir`, `env_first`),
  `apps/lightcraft/src/main.rs` + `logging.rs` (log names),
  `crates/ui-egui/src/control.rs` (exports dir), `crates/ui-egui/src/i18n.rs`
  (`RAYGUN_LANGUAGE`), `crates/gpu` (`RAYGUN_GPU*`), `crates/segment`
  (`RAYGUN_SAM3_*`), `crates/fetch` (token twin).
- Icons: `assets/app-icon/raygun.*` + hicolor `io.github.kao_offline.raygun.*`
  (regenerate with `packaging/icons.sh`; never reintroduce the lynx files).
- Installers: `packaging/windows/raygun.wxs`, `packaging/linux/*raygun*`,
  `packaging/macos/*`, `packaging/freebsd/*`, `.github/workflows/*`
  (artifact names, desktop id). Internal binary names stay
  `lightcraft`/`lightcraft-cli` on purpose.
- License strings: About dialog + `packaging/*/nfpm.yaml` + metainfo say
  AGPL-3.0-or-later, never MIT.

## Auto-update (Raygun's own, no reinstall)

- `update.check` (Help ▸ Check for Updates…, plus a weekly background check)
  compares this build against `github.com/kao-offline/raygun/releases/latest`.
- `update.download` selects the running app's OS and CPU architecture, verifies the GitHub asset SHA-256 digest, and records the download in `<config>/updates`.
- `update.apply` runs the downloaded MSI silently in place on Windows
  after the calling app exits (Raygun UpgradeCode: existing library files are untouched).
  Help > Check for Updates offers Download Update, then Restart and Install Update.
  The app saves first and asks about unfinished tasks or unsaved work. Windows may
  request administrator approval. The installer cannot reboot the computer; it
  records the actual exit code in `install-status.txt` and logs in `install.log`.
  `update.apply` returns `scheduled`, not a claim that installation succeeded.
  macOS/Linux currently download only; use the release package manually.
- Upstream LightCraft releases are **not** installed by the updater: it only
  follows Raygun releases, which is what makes merged upstream work safe to ship.
- Keep Raygun's `UpdatePoll` in `apps/lightcraft/src/main.rs` when integrating
  upstream changes: it delivers fresh check results directly to the update UI.

Windows 0.2.1 establishes a separate Raygun installation. Earlier packages shared LightCraft's MSI identity and cannot be distinguished reliably from stock LightCraft. They are not automatically removed; future Raygun packages upgrade only the new Raygun identity. Existing library folders remain usable.
