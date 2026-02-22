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

---

## Website Redesign + Logo Asset Sync (same session)

### Summary

Reworked the `/website` homepage to align with the root `README.md` positioning:

- Product identity: **Multi-Agent Terminal Office**
- Core promise: managing large AI-agent workflows from CLI
- Structure: hero, problem/solution cards, feature pillars, quick-install section

### Asset and Tooling Changes

- Added build-time asset sync for logo:
  - `../logo.svg` -> `website/public/logo.svg`
- Kept release asset sync for:
  - `install.sh`
  - `version.txt`
- Switched website lifecycle scripts to pnpm chain:
  - `build`, `dev`, `start` now run via `pnpm run ...`

### UX/UI Changes

- Introduced a stronger visual direction (non-default gradients, panel cards, staged reveal animations).
- Updated shared docs/home navigation branding to use Mato repo metadata and logo title.

### Further Refinement

- Homepage information architecture now mirrors README positioning:
  - Hero + core promise
  - Showcase section with real screenshot
  - Why Mato comparison block
  - Feature pillars
  - Shortcut philosophy table
  - Quick install section
- Added build-time screenshot sync:
  - `../docs/images/screenshot-0.png` -> `website/public/screenshot-0.png`
