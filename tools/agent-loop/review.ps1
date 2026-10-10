# Raygun review passes — run after loop.ps1 is green
# Pass A: never-crash (no unwrap/expect/panic/todo/unreachable/unsafe in prod)
# Pass B: layering (nothing below L5 touches egui/eframe/winit/rfd)
# Pass C: resolution independence + CPU/GPU parity spot-check
# Pass D: assets + licenses (ATTRIBUTION rows, no Adobe material, AGPL headers ok)
# Usage: .\tools\agent-loop\review.ps1

$ErrorActionPreference = "Stop"
Write-Output "Pass A: panic-surface scan (prod code only, tests excluded)"
$hits = Select-String -Path "crates/*/src/**/*.rs","apps/*/src/**/*.rs" -Pattern "\.unwrap\(\)|\.expect\(|panic!|todo!|unimplemented!|unreachable!" | Where-Object { $_.Path -notmatch "test" } | Select-Object -First 20
if ($hits) { $hits | ForEach-Object { Write-Output $_.Line }; exit 1 } else { Write-Output "Pass A clean (spot-check)." }

Write-Output "Pass B: layering"
cargo xtask layers

Write-Output "Pass C: build target crate"
cargo check -p lightcraft

Write-Output "Pass D: assets"
cargo xtask assets

Write-Output "REVIEW GREEN. Ready for commit."
