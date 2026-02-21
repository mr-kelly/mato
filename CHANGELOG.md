# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Placeholder for upcoming changes.

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

[Unreleased]: https://github.com/YOUR_USERNAME/mato/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/YOUR_USERNAME/mato/releases/tag/v0.4.0
[0.3.0]: https://github.com/YOUR_USERNAME/mato/releases/tag/v0.3.0
[0.2.0]: https://github.com/YOUR_USERNAME/mato/releases/tag/v0.2.0
[0.1.0]: https://github.com/YOUR_USERNAME/mato/releases/tag/v0.1.0
