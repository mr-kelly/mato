# Onboarding Office Name Simplification (2026-02-22)

## Summary

Adjusted onboarding default office naming to follow a simpler rule:

- Choose **one** identity source only.
- Prefer `username`; fallback to `hostname`.
- Capitalize the first letter.
- Use the format: `XX AI Office`.

## Code Changes

- Updated `src/client/onboarding_tui.rs` in `default_office_name()`.
- Removed combined `username@hostname` generation.
- Kept existing sanitization and length guard behavior.

## Examples

- `kelly` -> `Kelly AI Office`
- missing username + `devbox` hostname -> `Devbox AI Office`
- missing both -> `My AI Office`
