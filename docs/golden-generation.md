---
post_title: "gb-save-patcher: Golden Fixture Generation"
author1: "vulcandth"
post_slug: "gb-save-patcher-golden-fixture-generation"
microsoft_alias: "vulcandth"
featured_image: ""
categories:
  - tooling
tags:
  - parity-testing
  - save-files
  - wasm
ai_note: "AI-assisted drafting; reviewed and edited by a human."
summary: "How to generate golden patched saves using the upstream polished-save-patcher."
post_date: "2026-01-09"
---

## Goal

Generate **golden** patched outputs (v7 → v8/v9/v10, plus any fix patches) using the upstream
C++/Emscripten patcher, and store them under `tests/fixtures/`.

## Inputs

- Place input saves under `tests/fixtures/`.
- Keep them synthetic/sanitized.

## Option A: Build the upstream web patcher (Emscripten)

1. Install Emscripten (emsdk) and ensure `emcc` is on `PATH`.
2. Build the upstream patcher:

```powershell
Set-Location "third_party/polished-save-patcher"
make release
```

This produces `third_party/polished-save-patcher/build/index.html` plus the compiled
`polished_save_patcher.*` artifacts.

3. Serve the build output:

```powershell
Set-Location "third_party/polished-save-patcher/build"
python -m http.server
```

4. Open the local URL shown in the terminal, upload your v7 save, choose a target version,
   and download the patched output.

## Option B: Use an existing upstream build

If you already have a known-good build of the upstream patcher (e.g., a published GitHub
Pages URL), you can use that UI instead. The only requirement is that the produced bytes
match the upstream tool’s output.

## Where to put outputs

For each input save, generate outputs named like:

- `expected_v7_to_v8.sav`
- `expected_v7_to_v9.sav`
- `expected_v7_to_v10.sav`

Put these alongside the input save (or in a clearly named subfolder).

## Notes

- Parity is byte-for-byte on success; Rust-side error messages do not need to match upstream.
- If you want, we can later add a CI job that builds the upstream patcher as an oracle.
