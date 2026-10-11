# Raygun app icon

A **ray gun** silhouette in dark ink on a light tile: the user-supplied mark
(`assets/raygun.svg`), set into a 512 px square tile (`assets/app-icon/raygun.svg`).

## Geometry

`viewBox="0 0 512 512"`, gun scaled 0.85 and centered on a `#ECECEC` field.
macOS files (`.icns`, `raygun-macos-512.png`) add Apple's transparent margin (viewBox
widened to `-62 -62 636 636`); Windows and Linux files use the full tile so the
silhouette reads at 16-48 px.

## Files

| File | What |
|---|---|
| `raygun.svg` | canonical vector tile |
| `raygun-1024.png` | 1024 px render (store listings, docs) |
| `raygun-macos-512.png` | runtime window/Dock icon on macOS (embedded by `apps/lightcraft/src/main.rs`) |
| `raygun.icns` | macOS bundle icon (`CFBundleIconFile`), packed by `cargo xtask icns` |
| `raygun.ico` | Windows icon, 16-256 px, embedded in the exe by `apps/lightcraft/build.rs` |
| `hicolor/<n>x<n>/apps/io.github.kao_offline.raygun.png` | Linux icon theme, 16-512 px; the 256 px one is also the runtime window icon on Windows and Linux |
| `hicolor/scalable/apps/io.github.kao_offline.raygun.svg` | Linux scalable icon |

The app id is `io.github.kao_offline.raygun`: the Wayland app id, the `.desktop` file
(`packaging/linux/io.github.kao_offline.raygun.desktop`, `Icon=io.github.kao_offline.raygun`)
and the hicolor icon name. To install on Linux, copy `hicolor/` into
`/usr/share/icons/hicolor/` (or `~/.local/share/icons/hicolor/`) and the `.desktop`
file into `applications/`.

## Regenerate

Edit `raygun.svg`, then run `packaging/icons.sh` (needs `resvg`; the `.ico`/`.icns`
are packed by `cargo xtask ico` / `cargo xtask icns`). Every output is committed.
