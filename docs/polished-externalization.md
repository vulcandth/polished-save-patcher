---
post_title: "Polished Externalization Smoke Guide"
author1: "vulcandth"
post_slug: "polished-externalization"
microsoft_alias: "vulcandth"
featured_image: ""
categories:
  - rust
  - tooling
  - cli
  - web
  - wasm
tags:
  - architecture
  - crates
  - integration
  - logging
ai_note: "AI-assisted: drafted an externalization smoke guide based on the current shell architecture."
summary: "Notes for the Polished Crystal game crate repo, which depends on the shared gb-save-core/cli/web crates for CLI and WASM plumbing."
post_date: "2026-01-10"
---

## What this repo is

This repository is the **game-specific** crate for Polished Crystal (`gb-save-polished`).
It provides:

- game logic (version detection, migrations, fix patches)
- a CLI binary (implemented via `gb_save_cli::GameCli`)
- `wasm-bindgen` exports for browser usage

It depends on shared, game-agnostic crates from the sibling `gb-save-patcher` repo:

- `gb-save-core`: buffer/symbol/patch framework primitives
- `gb-save-cli`: game-agnostic CLI shell
- `gb-save-web`: WASM/JS conversion helpers and canonical JS outcome shape

## Repo layout

- `src/`: patcher implementation
- `resources/`: embedded symbols
- `tests/`: golden parity fixtures + tests
- `web/`: tiny local demo page (loads the WASM package produced by `wasm-pack`)

## Dependencies

This repo currently consumes `gb-save-core`, `gb-save-cli`, and `gb-save-web` via **path dependencies** to a sibling checkout.

Example (current) `Cargo.toml` shape:

```toml
[package]
name = "gb-save-polished"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib", "cdylib"]

[dependencies]
anyhow = "1"
gb-save-core = "<version>"
gb-save-cli = "<version>"
gb-save-web = "<version>"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
```

## CLI entrypoint

Create a small binary that implements `gb_save_cli::GameCli` and calls the generic runner.

Typical pattern:

- `detect_version(bytes: &[u8]) -> anyhow::Result<u16>`
- `patch(bytes: Vec<u8>, target: u16, dev_type: u8) -> anyhow::Result<Vec<u8>>`
- `patch_with_log(...) -> gb_save_cli::PatchOutcome`

The CLI shell handles:

- parsing arguments
- `--quiet` / `-v/-vv` verbosity filtering
- `--format human|json` output formatting
- `--color` and `NO_COLOR`

## WASM entrypoint

Expose `wasm-bindgen` functions from your game crate and return JS objects via `gb-save-web` helpers.

Recommended minimal API:

- `get_save_version(bytes: &[u8]) -> Result<u16, JsValue>`
- `patch_save(bytes: &[u8], target_version: u16, dev_type: u8) -> Result<Vec<u8>, JsValue>`
- `patch_save_with_log(bytes: &[u8], target_version: u16, dev_type: u8) -> JsValue`

For log-capable patching, return the canonical shape produced by:

- `gb_save_web::js::patch_outcome_to_js(bytes, logs, error)`

## JavaScript packaging expectations

Two common approaches:

1. `wasm-pack` (recommended for web consumers)

```powershell
wasm-pack build --target web
```

- Produces a `pkg/` directory suitable for direct import in modern browsers.
- Also supports `--target bundler` if your consumer is a bundler like Vite/Webpack.

2. Plain `cargo build` (lowest-level)

```powershell
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
```

- Produces a `.wasm` file, but you still need `wasm-bindgen`/packaging steps for ergonomic JS consumption.

## Stable JS outcome contract

UIs should treat these as stable:

- `ok: boolean`
- `error?: string`
- `bytes?: Uint8Array`
- `logs: Array<{ level: "info" | "warn" | "error", className: string, source: string, message: string }>`

UIs should treat the structure as stable, but presentation (CSS) is consumer-defined.

## Smoke checks

In the external repo:

1. CLI builds and runs:

```powershell
cargo run --bin gb-save-patcher -- --help
```

2. WASM builds (via wasm-pack):

```powershell
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --release --target web
```

3. JS integration sanity (browser):

- load the generated package (from `pkg/`)
- call `patch_save_with_log(...)`
- verify logs render with `className` and `level`

## Non-goals

- This guide does not mandate publishing a JS/TS wrapper package. If you have web consumers, a small wrapper can be useful, but it’s optional.
