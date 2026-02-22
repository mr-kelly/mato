# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Placeholder for upcoming changes.

## [0.6.0] - 2026-02-22

**Terminal Resilience + Website Refresh Release**

### Added
- Full website refresh aligned with README product positioning:
  - stronger home information architecture (hero, showcase, comparisons, features, shortcut philosophy, install)
  - build-time website asset sync for `logo.svg` and screenshot
  - navigation branding updates and project metadata cleanup
- Added regression tests for:
  - title sync using current terminal size
  - desk navigation spawning target active tab

### Changed
- `sync_tab_titles()` now fetches screen using current terminal size instead of `1x1`.
- Desk switching paths now proactively spawn the target active tab for faster first frame:
  - sidebar navigation (`↑/↓`)
  - jump-mode desk target selection
- Website scripts standardized around `pnpm` workflow in `/website`.

### Fixed
- PTY/process lifecycle reliability:
  - tabs auto-respawn when shell process exits instead of getting stuck
  - daemon request paths now ensure PTY is running before input/screen/mode operations
  - first-frame recovery improved by synchronous spawn + immediate screen retry when tab is missing
- Reduced terminal render/cursor drift risk for wide characters by handling spacer cells as zero-width placeholders.

## [0.5.1] - 2026-02-22

**Patch Release**

### Added
- Website analytics integration via `@vercel/analytics` in the Next.js root layout.

### Changed
- Onboarding default office name now uses a single identity token:
  - prefer `username`
  - fallback to `hostname`
  - formatted as `XX AI Office` with leading capitalization

## [0.5.0] - 2026-02-22

**Jump Mode + System Theme Polish Release**

### Added
- Onboarding office-name flow:
  - default office name prefilled from username/hostname
  - one-key start with template selection (`Enter`)
  - optional rename mode via `r` before starting
- Extended Jump labels from lowercase-only to `a-z/A-Z` (up to 52 targets).
- New docs:
  - `docs/changelog/2026-02-22_shell-experience-jump-mode-and-system-theme-polish.md`
  - updated operations guide in `README.md`
  - refreshed `docs/KEYBOARD_SHORTCUTS.md`

### Changed
- Renamed template identity:
  - `Power User` -> `Mato Creator Office`
  - onboarding/metadata copy now explicitly describes creator/builder setup
- Jump Mode label allocation strategy:
  - Content: balanced interleaving of tab/desk targets
  - Topbar: tabs first
  - Sidebar: desks first
- Jump target rendering and key mapping now use a unified target model (no label/key mismatch).
- System theme visual polish:
  - clearer active panel/tab/desk emphasis
  - focus/status badges aligned with theme behavior

### Fixed
- Fast typing flicker in active shell tab mitigated by cache/fallback behavior improvements.
- Jump popup transparency bleed removed (solid popup rendering path).
- Sidebar jump labels aligned with desk rows (offset fix).
- `w/a` no longer hijack letter jumps in Jump Mode.
- Jump focus arrows now follow explicit per-focus matrix:
  - Topbar: `←` Sidebar, `↓` Content
  - Sidebar: `→` Content, `↑` Topbar
  - Content: `←` Sidebar, `↑` Topbar
- Status indicator updated from `⚡` style to explicit connection semantics (`✓` / `·` / `•`).

📖 **[Full Release Notes →](docs/release-notes/RELEASE_NOTES_v0.5.0.md)**  
📖 **[Technical Summary →](docs/release-notes/RELEASE_SUMMARY_v0.5.0.md)**

## [0.4.0] - 2026-02-21

**Shell Experience & Performance Release**

### Added
- `mato --kill` to terminate daemon and related client/tab process tree, with summary output.
- Bracketed paste aware pipeline end-to-end (`Event::Paste` -> daemon -> PTY/emulator mode tracking).
- Input mode protocol (`GetInputModes` / `InputModes`) for mouse and bracketed paste capability checks.
- Richer terminal key encoding coverage (function keys, navigation keys, ctrl/alt combinations).
- Tab switch first-frame latency instrumentation.
- `RELEASE_NOTES_v0.4.0.md` and `RELEASE_SUMMARY_v0.4.0.md`.

### Changed
- Daemon screen pulling moved to background worker + cache model with adaptive polling and idle teardown.
- Screen fetch path now uses cached content for smoother rendering under load.
- Active-status polling is adaptive to active/idle states to reduce background overhead.
- Mouse passthrough in content area now respects application mouse mode.
- `--status` output now reflects current state model (offices/desks/tabs).
- Cursor style updates are sent only on shape changes (instead of every frame).

### Fixed
- Reduced active tab flicker during fast typing by avoiding transient blank-frame behavior.
- Improved resilience when daemon reports `tab not found` (self-heal respawn path retained).
- Lowered noise from expected short-lived disconnect patterns (broken pipe / close-miss cases).
- Better terminal resume behavior consistency for bracketed paste mode.

### Docs
- Updated release index for v0.4.0 as latest.

📖 **[Full Release Notes →](docs/release-notes/RELEASE_NOTES_v0.4.0.md)**  
📖 **[Technical Summary →](docs/release-notes/RELEASE_SUMMARY_v0.4.0.md)**

## [0.3.0] - 2026-02-21

**The Minimalist Release** 🎯

### Highlights
- Jump Mode navigation (Esc + letters)
- AI-agent-friendly key model (minimal shortcut conflicts)
- Unified keyboard shortcuts
- Terminal content preservation strategy on resize
- Ctrl+Z suspend/resume handling

📖 **[Full Release Notes →](docs/release-notes/RELEASE_NOTES_v0.3.0.md)**

## [0.2.0] - 2026-02-21

**Production Ready Release** ✅

### Highlights
- Daemon reliability and lifecycle improvements
- Initial testing infrastructure and modular refactor
- Onboarding templates and UX improvements

📖 **[Full Release Notes →](docs/release-notes/RELEASE_NOTES_v0.2.0.md)**  
📖 **[Technical Summary →](docs/release-notes/RELEASE_SUMMARY_v0.2.0.md)**

## [0.1.0] - 2026-02-20

**Initial Release** 🎉

### Highlights
- Daemon-based persistence baseline
- Desk/tab management in TUI
- Pluggable terminal backend structure

📖 **[Full Release Notes →](docs/release-notes/RELEASE_NOTES_v0.1.0.md)**

---

## Detailed Development History

- **[docs/changelog/](docs/changelog/)** - Dated development documents
- **[docs/release-notes/](docs/release-notes/)** - Release documentation
- **[AGENTS.md](AGENTS.md)** - AI-assisted development guide

## Version Links

[Unreleased]: https://github.com/mr-kelly/mato/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/mr-kelly/mato/releases/tag/v0.6.0
[0.5.1]: https://github.com/mr-kelly/mato/releases/tag/v0.5.1
[0.5.0]: https://github.com/mr-kelly/mato/releases/tag/v0.5.0
[0.4.0]: https://github.com/mr-kelly/mato/releases/tag/v0.4.0
[0.3.0]: https://github.com/mr-kelly/mato/releases/tag/v0.3.0
[0.2.0]: https://github.com/mr-kelly/mato/releases/tag/v0.2.0
[0.1.0]: https://github.com/mr-kelly/mato/releases/tag/v0.1.0
