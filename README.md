## polished-save-patcher

Polished Crystal save patcher (Rust) plus tooling for parity with the upstream C++ implementation.

This repository is a **single Rust crate** (`gb-save-polished`) containing:

- `src/`: save parsing, checksums, migrations, fix patches, and WASM exports
- `resources/`: embedded symbol resources
- `tests/`: parity tests and golden fixtures

This crate depends on shared, game-agnostic crates from the sibling repository `gb-save-patcher`:

- `gb-save-core`
- `gb-save-cli`
- `gb-save-web`

### Prerequisites

- Rust (stable)
- A checkout of `gb-save-patcher` **next to** this repo (because `Cargo.toml` uses `../gb-save-patcher/...` path dependencies)

Example directory layout:

```
<parent>/
	gb-save-patcher/
	polished-save-patcher/
```

### Build & test

```powershell
cargo test
```

### CLI (local)

This repo includes a small CLI binary that uses the shared CLI shell from `gb-save-cli`.

```powershell
cargo run --bin gb-save-patcher -- --help
```

### Local web (WASM)

There is a minimal no-framework browser demo in `web/`.

1. Install wasm tooling:

```powershell
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

2. Build the WASM package (creates `pkg/`):

```powershell
wasm-pack build --release --target web
```

3. Serve the repo root and open the demo page:

```powershell
python -m http.server 8000
```

Then visit `http://localhost:8000/web/`.

Shortcut: `./tools/web_dev.ps1` (builds WASM + serves on `:8000`).

### GitHub Pages

This repo includes a GitHub Actions workflow that builds the WASM package and publishes the demo to GitHub Pages on pushes to `main`.

The deployed site serves the demo at the Pages root (`/`). (`/web/` is also published as an alias.)

If Pages isn’t already enabled for the repo, set:

- Settings → Pages → Build and deployment → Source: **GitHub Actions**
