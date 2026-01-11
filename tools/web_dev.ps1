param(
  [int]$Port = 8000
)

$ErrorActionPreference = 'Stop'

Write-Host "Building WASM package (wasm-pack)..." -ForegroundColor Cyan

if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
  Write-Error "wasm-pack is not installed. Run: cargo install wasm-pack"
}

if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
  Write-Error "rustup is required to add wasm targets. Install Rust via rustup."
}

rustup target add wasm32-unknown-unknown | Out-Host

# Build into ./pkg (ignored by git)
wasm-pack build --release --target web | Out-Host

Write-Host "\nPreparing dist/ site using upstream gb-save-web UI..." -ForegroundColor Cyan

$UpstreamWww = Join-Path $PSScriptRoot "..\..\gb-save-patcher\crates\gb-save-web\www"
$UpstreamWww = (Resolve-Path $UpstreamWww -ErrorAction SilentlyContinue)
if (-not $UpstreamWww) {
  Write-Error "Could not find upstream UI at ..\\gb-save-patcher\\crates\\gb-save-web\\www. Check out gb-save-patcher next to this repo."
}

$Dist = Join-Path $PSScriptRoot "..\dist"
if (Test-Path $Dist) { Remove-Item -Recurse -Force $Dist }
New-Item -ItemType Directory -Path $Dist | Out-Null

Copy-Item -Recurse -Force (Join-Path $UpstreamWww "*") $Dist

@'
export const SITE_CONFIG = {
  game: {
    title: "Polished Crystal Save Patcher (Local)",
    subtitle: "Patches your save in your browser (no uploads).",
    githubUrl: "https://github.com/vulcandth/polished-save-patcher",
    githubLabel: "View source",
  },
  wasm: { modulePath: "./pkg/gb_save_polished.js" },
  ui: { showAdvancedMode: true },
  instructions: [
    "Back up your original save file somewhere safe.",
    "Choose your save file below.",
    "Pick a target version, then click Patch.",
    "Download the patched save and load it in your emulator/flashcart.",
  ],
  targetVersions: [
    { value: 8, label: "v8" },
    { value: 9, label: "v9" },
    { value: 10, label: "Latest (v10)" },
  ],
  acceptedExtensions: [".sav", ".srm"],
};
'@ | Set-Content -Encoding UTF8 (Join-Path $Dist "config.js")

# Copy wasm-pack output
Copy-Item -Recurse -Force (Join-Path $PSScriptRoot "..\pkg") (Join-Path $Dist "pkg")

Write-Host "\nServing dist/ on http://localhost:$Port/" -ForegroundColor Green

if (Get-Command python -ErrorAction SilentlyContinue) {
  python -m http.server $Port --directory $Dist
  exit 0
}

if (Get-Command py -ErrorAction SilentlyContinue) {
  py -m http.server $Port --directory $Dist
  exit 0
}

Write-Error "No Python found for a quick static server. Install Python or serve the repo root with another static file server."
