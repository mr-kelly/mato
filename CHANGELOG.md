# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.0] - 2026-02-22

### Added
- **Resize strategy**: configurable `resize_strategy` in config (`sync`/`fixed`). Default `sync` mode properly resizes PTY and emulator when terminal window changes, fixing vim/htop/lazygit rendering.
- **Per-tab spawn options**: `Spawn` protocol now supports optional `cwd`, `shell`, and `env` fields for per-tab working directory, shell, and environment variables.
- **Compatibility smoke tests**: `tests/compat/smoke_test.sh` for automated TUI app compatibility validation.

### Changed
- AlacrittyEmulator now implements real `resize()` using `term.resize()` instead of no-op.
- Daemon resize handler respects config strategy instead of always ignoring resize.

### Performance
- **Persistent write connection**: Input/Paste/Resize/Scroll messages now reuse a single persistent Unix socket connection instead of opening a new connection per keystroke. Reduces per-keystroke overhead from ~1-2ms to ~0.01ms.
- **Keystroke-triggered screen refresh**: Screen worker wakes immediately via Condvar after input instead of waiting up to 40ms polling cycle. Combined with adaptive main loop (5ms poll after keystroke, 30ms active, 200ms idle), total input-to-echo latency reduced from ~40-120ms to ~5-15ms.
- **Zero-copy screen reads**: Eliminated `try_clone()` (dup syscall) per screen fetch frame. Reads directly into reusable 256KB buffer with 8KB chunk I/O.
- **Batch event drain**: Main loop now drains ALL pending crossterm events before rendering instead of one-per-frame.
- **Post-wake burst**: After keystroke wake, screen worker does 2 rapid re-fetches at 5ms to catch echo faster.
- **Binary screen protocol (MessagePack)**: Screen responses now use MessagePack instead of JSON. 8x smaller payloads (400KB → 50KB per frame for 80x24). Other messages stay JSON for compatibility.
- **Screen hash dedup**: Daemon tracks content hash per connection. When screen hasn't changed, sends tiny `ScreenUnchanged` response instead of full frame. Eliminates redundant serialization/transfer when idle.
- **Push-mode screen updates**: Client subscribes to tab via `Subscribe` message. Daemon pushes screen updates when PTY has output instead of client polling. Eliminates polling overhead entirely — updates arrive within ~2ms of PTY output.
- **Binary input protocol**: Input/Paste messages use MessagePack framing over the subscribe connection for single-handler echo path. Eliminates JSON serialization overhead.
- **Conditional render**: Main loop tracks `screen_generation` counter from push mode. Skips `terminal.draw()` when screen content hasn't changed. Reduces idle CPU usage.
- **Echo spin**: After content keystroke, main loop spins up to 3ms checking for screen_generation bump to catch echo in the same render frame.
- **Adaptive poll**: Main loop poll timeout: 1ms after input, 8ms follow-up, 16ms active, 100ms idle.
- **poll(2)-based socket reads**: Worker thread uses `libc::poll()` to check socket readability before calling `read_response()`. When no data available, sleeps only 200µs instead of blocking up to 5ms. 25x faster channel drain cycle.
- **Skip coalesce after input**: Daemon push loop skips 500µs coalesce check when the trigger was interactive input (Input/Paste). Bulk output still coalesces normally.
- **Pre-allocated push frame buffer**: Daemon push loop reuses a single buffer for frame construction instead of allocating ~50KB per push.
- **Incremental screen updates (ScreenDiff)**: Daemon tracks last-sent screen and compares line-by-line. When ≤50% lines changed, sends only changed lines via new `ScreenDiff` message (~1-2KB) instead of full screen (~50KB). Falls back to full screen for large changes (resize, bulk output). 25-50x smaller payloads for interactive typing.

### Fixed
- **ESC passthrough**: ESC now passes through to shell applications (vim, fzf, htop) in Content focus. Use double-ESC (within 300ms) to enter Jump Mode from Content. Non-Content focus unchanged.
- **Rename popup visibility/latency in Sidebar/Topbar**: pressing `r` could set rename state without immediate popup visibility. Main-loop key handling now refreshes UI timing for any Main-screen key, so rename popup appears immediately without requiring an extra click/focus change.
- **Jump Mode `r` behavior consistency**: in Jump Mode, `r` now correctly maps by focus (`Sidebar`/`Topbar` => Rename, `Content` => Restart terminal) and supports both `r`/`R`.

## [0.7.1] - 2026-02-22

**Onboarding State/Exit Semantics + Terminal Cleanup Refinement**

### Added
- Onboarding mode split for clear behavior boundaries:
  - `FirstRun` onboarding (startup without `state.json`)
  - in-app `New Office` onboarding
- Main-loop screen state routing (`Main` / `Onboarding`) for runtime onboarding under a single terminal owner.

### Changed
- Runtime `New Office` onboarding now runs inside the main event loop/state machine instead of a nested terminal loop.
- Onboarding help hints are mode-specific:
  - first-run shows `q Quit`
  - in-app shows `Esc Back`
- Onboarding template order updated:
  - `Start from Scratch` moved to first position
  - `Mato Creator Office` moved to second position
- `Start from Scratch` template expanded from minimal single-desk setup to:
  - 3 desks
  - 2 tabs per desk
- UI code quality cleanup:
  - simplified style-branch logic in `src/client/ui.rs`
  - daemon status spinner parity check uses `.is_multiple_of(2)`
  - removed unnecessary cast in alacritty emulator path
  - added `Default` implementation for `TerminalGuard`

### Fixed
- Residual TUI/shell artifacts on onboarding transitions and exits by enforcing explicit clear/reposition cleanup (`Clear + MoveTo(0,0)` before leaving alt-screen).
- First-run onboarding cancellation path no longer drops into inconsistent terminal state.
- Startup onboarding now clears the alternate screen before first frame, avoiding visible shell residue from pre-launch content.

## [0.7.0] - 2026-02-22

**Terminal Rendering Overhaul + Cursor/Bell Pipeline Release**

### Added
- Bell forwarding from terminal emulator events to host terminal output.
- Full terminal cell attribute transport/render support additions:
  - `reverse` (INVERSE)
  - `dim`
  - `strikethrough`
  - `hidden`
  - extended underline mode mapping
  - underline color support
  - zero-width combining character handling
  - explicit cell display width metadata
- Cursor telemetry during debugging (`cursor-debug`) for reproducible diagnosis across CLI apps.

### Changed
- Alacritty screen extraction path moved to renderable display iteration model.
- Cursor shape handling switched to DECTCEM-aware renderable cursor semantics.
- Cursor rendering approach simplified toward software-overlay behavior in the content layer.
- Terminal mode detection for bracketed paste/mouse now reads native `TermMode` flags (removed manual byte-sequence scanning and tail buffering).
- vt100 adapter updated to populate expanded screen-cell fields consistently with the new render model.

### Fixed
- Claude Code cursor visibility failure caused by missing INVERSE attribute propagation.
- Stale bottom cursor artifact by aligning cursor visibility handling with hidden cursor mode.
- Terminal title reset behavior now clears stale titles on `ResetTitle`.

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

[Unreleased]: https://github.com/mr-kelly/mato/compare/v0.8.0...HEAD
[0.8.0]: https://github.com/mr-kelly/mato/releases/tag/v0.8.0
[0.7.1]: https://github.com/mr-kelly/mato/releases/tag/v0.7.1
[0.7.0]: https://github.com/mr-kelly/mato/releases/tag/v0.7.0
[0.6.0]: https://github.com/mr-kelly/mato/releases/tag/v0.6.0
[0.5.1]: https://github.com/mr-kelly/mato/releases/tag/v0.5.1
[0.5.0]: https://github.com/mr-kelly/mato/releases/tag/v0.5.0
[0.4.0]: https://github.com/mr-kelly/mato/releases/tag/v0.4.0
[0.3.0]: https://github.com/mr-kelly/mato/releases/tag/v0.3.0
[0.2.0]: https://github.com/mr-kelly/mato/releases/tag/v0.2.0
[0.1.0]: https://github.com/mr-kelly/mato/releases/tag/v0.1.0
