---
post_title: "gb-save-polished: Test Fixtures"
author1: "vulcandth"
post_slug: "gb-save-polished-test-fixtures"
microsoft_alias: "vulcandth"
featured_image: ""
categories:
  - rust
tags:
  - parity-testing
  - save-files
ai_note: "AI-assisted drafting; reviewed and edited by a human."
summary: "How to organize golden save fixtures for parity tests."
post_date: "2026-01-09"
---

## Fixtures

This folder is intended for **golden** inputs/outputs used by parity tests.

### Where to put files

- Place fixtures under `tests/fixtures/`.
- Keep files synthetic/sanitized (no personal data).

### Layout contract (current)

- Each fixture set is a directory under `tests/fixtures/`, for example:
  - `MrKat-Save/`
  - `ProtoBlues-Save/`
- Each fixture set contains:
  - `v7/` with exactly one input `.sav` (the pre-migration save)
  - `v8/patched_save.sav` (expected output after v7→v8)
  - `v9/patched_save.sav` (expected output after v7→v8→v9)
  - `v10/patched_save.sav` (expected output after v7→v8→v9→v10)

Parity tests should discover fixture sets automatically and compare patched output bytes to the corresponding `patched_save.sav` golden.

### Naming

- Input filenames in `v7/` are not significant; only the `.sav` extension matters.
- Expected output filenames are fixed:
  - `v8/patched_save.sav`
  - `v9/patched_save.sav`
  - `v10/patched_save.sav`

### Notes

- Parity means the patched output bytes must match the upstream patcher byte-for-byte.
- When you’re ready, tell me which fixtures you want to start with, and I’ll wire them into integration tests.
