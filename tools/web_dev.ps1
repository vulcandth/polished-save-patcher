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

Write-Host "\nServing repo root on http://localhost:$Port/web/" -ForegroundColor Green

if (Get-Command python -ErrorAction SilentlyContinue) {
  python -m http.server $Port
  exit 0
}

if (Get-Command py -ErrorAction SilentlyContinue) {
  py -m http.server $Port
  exit 0
}

Write-Error "No Python found for a quick static server. Install Python or serve the repo root with another static file server."
