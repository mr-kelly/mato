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

---

## Terminal Stability and Cursor Fixes (same session)

### Issues Addressed

1. Tabs became unusable after shell `exit` (dead PTY stayed attached).
2. Some tabs occasionally opened without a usable terminal session.
3. Cursor misalignment observed in `claude` (while `gemini-cli`/`codex-cli` were OK).

### Changes

- Added PTY liveness checks and auto-respawn in `PtyProvider`:
  - detect exited child via `try_wait()`
  - respawn shell when tab is revisited or receives input
  - fallback to `/bin/sh` if configured shell spawn fails
  - removed panic-on-spawn paths (`expect`) in favor of recoverable logging/retry
- Daemon now proactively ensures PTY is running before handling:
  - `Input`
  - `Paste`
  - `GetInputModes`
  - `GetScreen`
  - `Scroll`
- Updated title-sync polling to avoid 1x1 screen fetch side effects:
  - `sync_tab_titles()` now requests current terminal size instead of `get_screen(1, 1)`
- Improved desk-switch reliability:
  - sidebar desk navigation now proactively spawns the target desk's active tab
  - desk jump (`Esc` jump to desk) also proactively spawns target active tab
- Improved first-frame recovery when daemon returns `tab not found`:
  - client now performs synchronous `Spawn` and immediate retry `GetScreen` once
- Added rendering fix for wide-char spacer cells in Alacritty emulator output:
  - treat spacer cells as zero-width placeholders
  - prevents width accumulation drift that can manifest as cursor offset in richer CLIs

### Test Coverage Added

- Added regression test ensuring `sync_tab_titles()` uses current terminal size instead of 1x1.
