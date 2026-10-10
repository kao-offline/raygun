# Raygun agent loop — check → fix → review until green
# Usage: .\tools\agent-loop\loop.ps1 [-MaxPasses 5] [-Target lightcraft]
# Each pass runs: fmt check, clippy, unit tests for touched crates, layers, assets.
# Failures are appended to progress.json with a RETURN directive so the next
# agent pass knows exactly what to fix (no slop, no guessing).
param(
  [int]$MaxPasses = 5,
  [string]$Target = "lightcraft"
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
Set-Location $root
$progressFile = Join-Path $PSScriptRoot "progress.json"

function Write-ProgressEntry($pass, $stage, $ok, $detail) {
  $entry = [ordered]@{
    time = (Get-Date).ToString("o"); pass = $pass; stage = $stage
    ok = $ok; detail = ($detail | Out-String).Substring(0, [Math]::Min(4000, ($detail | Out-String).Length))
  }
  $log = @()
  if (Test-Path $progressFile) { try { $log = Get-Content $progressFile -Raw | ConvertFrom-Json } catch { $log = @() } }
  $log += $entry
  $log | ConvertTo-Json -Depth 4 | Set-Content $progressFile
}

$pass = 1
while ($pass -le $MaxPasses) {
  Write-Output "=== RAYGUN LOOP pass $pass/$MaxPasses (target: $Target) ==="
  $stages = @(
    @{ name = "fmt"; cmd = "cargo fmt --check" },
    @{ name = "clippy"; cmd = "cargo clippy -p $Target -- -D warnings" },
    @{ name = "test"; cmd = "cargo test -p $Target" },
    @{ name = "layers"; cmd = "cargo xtask layers" },
    @{ name = "assets"; cmd = "cargo xtask assets" }
  )
  $failed = $null
  foreach ($s in $stages) {
    Write-Output "-- stage: $($s.name): $($s.cmd)"
    try {
      $out = Invoke-Expression $s.cmd 2>&1 | Out-String
      Write-Output $out
      Write-ProgressEntry $pass $s.name $true $out
    } catch {
      $msg = $_.Exception.Message + "`n" + ($_.InvocationInfo.PositionMessage)
      Write-Output "FAILED $($s.name): $msg"
      Write-ProgressEntry $pass $s.name $false $msg
      $failed = @{ stage = $s.name; pass = $pass; hint = "RETURN: fix $($s.name) failure, re-run loop.ps1. Keep diff minimal, pure Rust, no unwrap/expect/panic/unsafe in prod." }
      break
    }
  }
  if (-not $failed) { Write-Output "GREEN: all stages passed on pass $pass."; break }
  Write-Output ("RETURN directive: " + ($failed | ConvertTo-Json -Compress))
  $pass++
}
if ($pass -gt $MaxPasses) { Write-Output "RED: max passes hit. See tools/agent-loop/progress.json."; exit 1 }
