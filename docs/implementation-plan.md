---
post_title: "gb-save-patcher: Rust Port Implementation Plan"
author1: "vulcandth"
post_slug: "gb-save-patcher-rust-port-plan"
microsoft_alias: "vulcandth"
featured_image: ""
categories:
  - rust
tags:
  - gameboy
  - save-files
  - wasm
  - cli
  - parity-testing
ai_note: "AI-assisted drafting; reviewed and edited by a human."
summary: "Checklist plan for a Rust port of polished-save-patcher with byte-for-byte parity, CLI + fully client-side web app, and an extensible patch framework."
post_date: "2026-01-08"
---

## Scope and Success Criteria

- [ ] Confirm the target:
  - [x] Polished Crystal saves
  - [x] Migrations v7→v8→v9→v10
  - [x] Fix patch infrastructure (`dev_type`) for future patches
- [x] Define “parity” precisely: output save bytes must match the upstream patcher byte-for-byte for the same input save + parameters (target version, `dev_type`).
- [x] Define supported IO:
  - [x] CLI: read from a file path, write to an output path.
  - [x] Web: read from a browser File, return patched bytes (no server).
- [ ] Define supported commands:
  - [x] `get-save-version` equivalent.
  - [x] `patch-save` equivalent (sequential migrations when `dev_type == 0`).
  - [x] `fix-save` (or `patch-save --dev-type N`) for one-off fixes.

## Repo Layout (Single Crate)

- [x] Implement a single Rust crate at the repo root:
  - [x] `src/`: Polished Crystal-specific helpers + patch implementations.
  - [x] `resources/`: embedded `.sym` resources.
  - [x] `tests/`: parity tests + golden fixtures.

Note: this crate depends on shared, game-agnostic crates from a sibling `gb-save-patcher` repo during local development.
  - [ ] Optional: `xtask/` for reproducible build steps (CI + local).
- [x] Keep `main.rs` thin: parse args, call library functions, render errors.
- [x] Add consistent error types:
  - [x] Library crates use `thiserror` + typed errors.
  - [x] Binaries (`cli`, `web`) may use `anyhow` for top-level context.

## Data Model: Save Binary Access

- [x] Implement a `SaveBinary` equivalent in Rust:
  - [x] Backed by `Vec<u8>`.
  - [x] Safe bounds checks with helpful error messages (no panics in library).
  - [x] Little-endian and big-endian word accessors.
  - [x] Read/write helpers for bytes, words, bit operations.
- [x] Decide how to represent addresses and ranges:
  - [x] Newtype `Address(u32)` and `Size(u32)` to avoid mixups.
  - [x] `Range { start: Address, end: Address }` with explicit inclusive/exclusive semantics.
- [x] Match upstream semantics:
  - [x] Save version is a big-endian word at `SAVE_VERSION_ABS_ADDRESS`.
  - [x] Copy/clear/fill operations behave identically to upstream (including overlap behavior, if any).

## Symbol Database (.sym) Parity

- [x] Re-implement upstream `SymbolDatabase` behavior, not just “a symbol parser”:
  - [x] Parse exactly the same file format as upstream uses.
  - [x] Preserve symbol lookup rules: missing symbol handling, duplicates, and whitespace quirks.
  - [x] Store as `HashMap<String, Address>` plus any needed metadata.
- [x] Add test vectors:
  - [x] “Known symbol exists” lookups.
  - [x] “Unknown symbol” behavior matches upstream error/failure.
  - [x] Cross-version sanity: the same symbol name can map to different addresses per version.
- [x] Resource strategy:
  - [x] Decide whether to embed `.sym` files in the polished crate via `include_str!` / `include_bytes!`.
  - [x] Keep a clear update path when upstream `.sym` files change.

## Checksums and Validation Gates

- [x] Port checksum algorithms byte-for-byte:
  - [x] Main save checksum: `calculateSaveChecksum(symbols["sGameData".."sGameDataEnd"])`.
  - [x] Backup save checksum: `calculateSaveChecksum(symbols["sBackupGameData".."sBackupGameDataEnd"])`.
  - [x] Newbox checksum routines: extract, validate, compute, and write, exactly matching upstream.
- [x] Port migration “gates” (must fail when upstream fails):
  - [x] Reject saves where checksum validation fails.
  - [x] Enforce “player must be in Pokémon Center 2F” for version migrations.
  - [x] Preserve special-case disallow/allow logic (e.g., the Shamouti PC restriction in 7→8).

## Patch Framework and Extensibility

- [x] Define an internal patch trait with explicit metadata:
  - [x] `id` (stable string), `from_version`, `to_version`, `kind` (migration vs fix).
  - [x] `apply(&self, save: &mut SaveBinary, symbols: &SymbolDatabase) -> Result<()>`.
- [ ] Add a registry:
  - [x] Ordered migration chain (7→8→9→10).
  - [x] Separate fix-patch registry keyed by `dev_type`.
  - [x] `resolve_plan(current_version, target_version)` returns a sequence of patches.
- [ ] Extensibility hooks:
  - [ ] Allow multiple “games” by introducing a `GameDefinition` trait (symbols + patch registry + known constraints).
  - [ ] Keep “core” crate GB-agnostic (only binary primitives + framework).

## Porting Strategy for Polished Crystal

- [x] Start with scaffolding and parity tests before porting huge mapping tables.
- [x] Port in dependency order:
  - [x] `SaveBinary` and helpers.
  - [x] `SymbolDatabase`.
  - [x] Common patch functions (checksum, copy/clear, bit/flag helpers).
  - [x] Version gate checks (location, warp ID validation, etc.).
- [ ] Port patch implementations:
  - [x] v7→v8 migration (includes newbox migration, mon struct conversions, item/key item/event flag remaps).
  - [x] v8→v9 migration (event flag remap table, warp validation/reset, key item remap).
  - [x] v9→v10 migration (text speed reversal, Ralph Magikarp quirk, mail conversion, event flag remap).
  - [ ] Future fix patches (no longer porting legacy `dev_type` fixes 1–6).
- [x] Keep mapping tables data-driven:
  - [x] Store large remap tables as `&'static [(u16, u16)]` (or `phf` if needed), with deterministic iteration.
  - [x] Avoid “clever” compression until parity is proven.

## CLI (gb-save-cli)

- [x] Use `clap` for a stable CLI surface:
  - [x] `gb-save-patcher version <path>`.
  - [x] `gb-save-patcher patch --in <path> --out <path> --target <7|8|9|10> [--dev-type N]`.
  - [ ] Optional: `--in-place` only if explicitly desired (default should be safe).
- [ ] Error UX:
  - [ ] Print concise, actionable errors (e.g., “checksum mismatch”, “not in Pokémon Center 2F”).
  - [ ] Exit codes: non-zero for failures.

## Web (WASM) App (Fully Client-Side)

- [x] Keep web implementation minimal and dependency-light:
  - [x] Use `wasm-bindgen` (UI implemented in plain JS; no framework).
  - [x] Provide a JS-friendly API: `get_save_version(bytes) -> u16`, `patch_save(bytes, target_version, dev_type) -> Vec<u8>`.
- [x] Provide a simple static UI:
  - [x] File upload + “Patch” button.
  - [x] Show detected version and validation errors.
  - [x] Download patched save as a file.
- [ ] GitHub Pages deployment:
  - [ ] Build WASM + static assets into a `dist/` folder.
  - [ ] Add a GitHub Actions workflow to publish `dist/` to `gh-pages`.

## Parity Test Strategy (Critical)

- [x] Add “golden” parity tests at the library level:
  - [x] Input fixture save(s) per version (7/8/9), plus expected outputs for each target migration.
  - [ ] Separate fixtures for future fix patches (`dev_type`) as-needed.
- [ ] Define fixture policy:
  - [ ] Use synthetic/minimal saves (no personal data) and keep them small/curated.
  - [ ] Document how fixtures were produced (exact upstream commit + command-line used).
- [ ] Testing layers:
  - [x] Unit tests for checksum + newbox checksum.
  - [x] Unit tests for mapping tables (spot checks + invariants).
  - [ ] Integration tests: end-to-end patching produces exact bytes.

## Tooling and CI

- [ ] Add GitHub Actions workflows (optional):
  - [ ] `cargo fmt -- --check`.
  - [ ] `cargo clippy --all-targets --all-features -- -D warnings`.
  - [ ] `cargo test`.
  - [ ] Build web target (via `wasm-pack`) and ensure it compiles.
- [x] Add a local web dev script:
  - [x] `tools/web_dev.ps1` builds WASM and serves the static UI.
- [ ] Add local developer scripts (optional `xtask`):
  - [ ] `xtask ci` to run fmt/clippy/test.
  - [ ] `xtask web-build` to produce `dist/`.

## Milestones (Recommended)

- [x] M0: Repo scaffolding (single crate) + tests green.
- [x] M1: `SaveBinary` + `.sym` parser + checksum functions with unit tests.
- [x] M2: v9→v10 port + parity fixture.
- [x] M3: v8→v9 port + parity fixture.
- [x] M4: v7→v8 port + parity fixture.
- [ ] M5: Future fix patches + fixtures.
- [ ] M6: Web UI polished + GitHub Pages publish workflow.

## Open Questions (Decide Early)

- [x] Fixture source-of-truth: commit golden outputs; optionally add an upstream-oracle job later.
- [x] Mirror upstream failures loosely; prioritize byte-for-byte output parity on success.
- [x] Use a compile-time registry for extensibility (plugin system deferred).
