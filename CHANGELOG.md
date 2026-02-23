# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **Restart Terminal shortcut changed from `r` to `x`**: In Jump Mode with Content focus, the restart terminal action now uses `x` key instead of `r` to avoid accidental triggers when attempting to rename tabs/desks. This prevents confusion between rename (`r` in Sidebar/Topbar) and restart operations.

### Fixed
- **Theme lost when connecting via mosh**: `Color::Rgb` escape sequences (`\x1b[38;2;R;G;Bm`) were emitted unconditionally. mosh ≤ 1.3.x silently drops them; even mosh 1.4+ requires `COLORTERM=truecolor` to be propagated. Added automatic 256-color fallback: when `COLORTERM` is not `truecolor`/`24bit`, each theme color is mapped to the nearest xterm-256 indexed color (6×6×6 cube + 24-step grayscale ramp) via `Color::Indexed`. This means themes work correctly in mosh without any configuration — colors are a close approximation rather than exact RGB. A startup toast is shown when COLORTERM is absent and a custom theme is active, informing the user they can set `COLORTERM=truecolor` for exact colors.
- **Toast rendering panic in narrow terminals**: `draw_toast` computed `toast_area.width = msg.len() + 4` without clamping to `frame.area().width`. On terminals narrower than the toast message, the buffer access was out-of-bounds. Fixed by clamping `width = msg_len.min(area.width)`.
- **ESC key silently dropped when followed by another key within 300ms**: The ESC double-press detection window held the ESC in `last_content_esc`. When a non-ESC key was pressed within 300ms, `flush_pending_content_esc()` was called but its `>= 300ms` guard prevented the flush — the ESC was silently dropped. This broke ESC+key combos (Alt sequences in readline, ESC followed by a vim command after exiting insert mode). Fixed in `input.rs`: on any non-ESC keypress, the pending ESC is now forwarded immediately via `last_content_esc.take()` and `pty_write(b"\x1b")`, bypassing the timer guard. The timer-based path (`flush_pending_content_esc` in the event loop) is unchanged.
- **Focus events (FocusIn/FocusOut) not sent on tab/desk switch within Content focus**: `sync_focus_events()` fires only when mato's `focus` field changes (e.g., Content→Sidebar). When switching tabs via Alt+1-9 or Jump Mode while staying in Content focus, the focus field stays at `Content` and `sync_focus_events()` does not fire — inner TUI apps with focus tracking enabled (vim, helix, kakoune) never receive `\x1b[O` (FocusOut) for the old tab or `\x1b[I` (FocusIn) for the new tab. Fixed by adding `pty_send_focus_event(focus_in: bool)` helper to `App` (guards on `focus == Content` and `focus_events_enabled()`) and calling it before and after every desk/tab switch that keeps focus in Content: Alt+1-9 in `input.rs`, and both 't' and 'b' paths in `handle_jump_selection`.
- **Multi-client screen sync broken when clients have different window sizes**: When a smaller client (e.g., phone at 25 rows) subscribed to a PTY spawned by a larger client (e.g., PC at 40 rows), `get_screen(25, cols)` in the alacritty emulator returned the TOP 25 rows of the 40-row PTY. The shell and cursor are always at the BOTTOM of the PTY (row 39), so the smaller client saw stale empty rows and the cursor was clamped — typed characters were invisible, scrolling didn't propagate. Fixed in `src/emulators/alacritty_emulator.rs` and `src/emulators/vt100_emulator.rs`: `get_screen(rows, cols)` now computes `row_offset = screen_lines - render_rows` and shows the BOTTOM `rows` rows. All cell indices and the cursor row are adjusted by `row_offset`. When sizes match (`row_offset = 0`) behavior is unchanged.
- **Multiple clients fight over PTY size (resize storm)**: Default `resize_strategy = Sync` caused every client's window resize to propagate to the PTY. With two clients of different sizes (phone + PC), each client's keyboard-popup resize sent a `Resize` to the daemon — the PTY oscillated between 25 and 40 rows, sending SIGWINCH to the shell repeatedly and causing TUI apps (vim, htop) on both clients to reflow constantly. Fixed: added `subscriber_count: Arc<AtomicUsize>` to `PtyProvider`; the daemon push loop increments it on Subscribe and decrements on client disconnect. The `Resize` handler in the push loop only resizes the PTY when `subscriber_count <= 1` (sole subscriber). With multiple subscribers, `sub_rows`/`sub_cols` still update so `get_screen` returns the correct bottom-rows view for each client's size — but the PTY itself stays at its original size.

### Fixed
- **Terminal content unstable on Android keyboard show/dismiss (cursor stuck in middle)**: When the keyboard raises the screen shrinks, then keyboard dismisses and the screen grows back. The client fires a fire-and-forget `Resize` to the daemon while simultaneously sending a sync `GetScreen` on a separate connection — these can be reordered. If `GetScreen` arrives before `Resize`, the daemon returns the old (smaller) PTY content (e.g. 25 lines) for the new display height (e.g. 40 rows). With top-alignment (`row_base = 0`) this renders the 25 content lines at the top with 15 empty rows below, leaving the cursor visually "stuck in the middle" until the push loop delivers the correctly-sized screen. Fixed by switching to **universal bottom-alignment**: `row_base = ih - screen_rows` for both normal and copy mode. When content is correctly sized (`screen_rows == ih`), `row_base = 0` and there is no change. When there is a transient size mismatch, the content pins to the bottom, keeping the cursor near the visual bottom regardless of when the PTY processes the resize.
- **Last line of terminal content not rendered after window resize**: after a window expand the client's `DaemonProvider` would call `cache_screen(content, new_rows, cols)` with content that only had `old_rows` lines (the PTY had not yet been resized when the synchronous `GetScreen` fallback ran). `cached_screen(new_rows, cols)` then matched on the row key but returned a short content, leaving the bottom row(s) permanently blank until the next push-loop full-screen arrived. Fixed by (a) adding a line-count consistency check in `cached_screen` so mismatched entries are rejected, and (b) skipping `cache_screen` in the sync fallback when `content.lines.len() != rows`. Added 4 unit tests for cache integrity.
- **Initial PTY spawn size off-by-one**: `term_rows` was initialised to `H-5` (comment said "topbar(3) + 2 borders") but missed the 1-row status bar, giving `H-5` instead of `H-6`. This caused an unnecessary SIGWINCH resize on every startup. Fixed to `H-6`.
- **`navy` theme not implemented**: `"navy"` was listed in `BUILTIN_THEMES` but had no match arm in `builtin()`, silently falling through to the `_` wildcard (system/transparent theme). Added proper deep-navy palette: bg `#0A0F23`, sky-blue + gold accents.
- **One Dark theme surface is lighter and more blue-biased than the editor background**: `surface` was `[44,50,60]` — lighter *and* more blue than `bg` `[40,44,52]`, opposite of authentic One Dark (sidebar `#21252B` = `[33,37,43]` is darker than editor). Fixed `surface` to `[33,37,43]` and `border` to `[55,60,71]` to match the real One Dark UI palette and reduce navy appearance.

### Tests
- **252 tests** (+~53 new tests, 0 failures):
  - `app_tests`: +29 new — `RenameState` unicode/cursor editing (emoji, CJK, backspace, delete, home/end, boundary clamps), `select_desk` dirty tracking, `switch_office` behavior, `show_toast`, `has_active_tabs`, `flush_pending_content_esc` stale/recent guard, jump label exclusions per focus, jump label uniqueness, tab-switch timing roundtrip, `handle_jump_selection` (desk, tab, invalid char)
  - `theme_tests`: +12 new — all 12 builtin themes have valid RGB, all themes are implemented (no fallthrough to `_`), One Dark surface darker than bg, Gruvbox warm, Navy blue-dominant, theme merge with partial overrides, TOML roundtrip
  - `protocol_tests`: +8 new — scroll delta roundtrip, subscribe message roundtrip, ScreenDiff bell+focus_events both true, InputModes all combinations, full Screen msgpack roundtrip, Error roundtrip, GetInputModes roundtrip, ScreenDiff cell attributes (italic/underline/reverse)
  - `daemon_provider` unit: +4 new — cache rejects mismatched line count, accepts consistent entry, stores correct size, rejects col mismatch



### Added
- **Toast notification system**: `app.toast: Option<(String, Instant)>` field drives bottom-right floating messages. Visible for 3s with binary DIM fade after 2s. Event loop drives `dirty=true` while active so toast auto-expires without needing user input. Triggered on: desk created, desk closed (confirmed), desk/tab/office renamed.
- **Jump Mode background dim**: entering Jump Mode now applies `DIM` modifier to every cell in the buffer, visually suppressing the background to make jump labels stand out.
- **Google Analytics (GA4) integration for website**: added via Next.js official `@next/third-parties` with configured measurement id.

### Fixed
- **Focus sequences (`^[[I`/`^[[O`) shown as literal characters on tab switch**: `sync_focus_events()` now gates on `focus_events_enabled` from the screen cache — sequences are only sent to a PTY that has explicitly enabled focus tracking via `\x1b[?1004h`. If the shell/app hasn't opted in, focus transitions are silently skipped.
- **Bell rings continuously after a single BEL**: cached `ScreenContent.bell = true` was never cleared after being consumed by the renderer, causing `pending_bell` to be set on every frame. Bell in the daemon provider screen cache is now cleared immediately after being read once (consume-on-read).
- **`focus_events_enabled` not pushed to client on mode change**: `meta_changed` check in the daemon's ScreenDiff push loop did not include `focus_events_enabled`, so a PTY enabling/disabling focus tracking would not trigger a diff push — clients could become stale. Added to `meta_changed` in `src/daemon/service.rs`.
- **Spinner freezes when focus is on terminal with no activity**: `ui_changed` check did not include spinner timing, so when there were no PTY output changes, no dirty flag, and no recent input, `terminal.draw()` was skipped entirely — spinner froze until the next keypress. Fixed by adding `app.has_active_tabs() && app.spinner_needs_update()` to the `ui_changed` condition. Spinner now drives its own redraws at 80ms intervals whenever there are active tabs, regardless of user input.
- **Toast fade DIM applied to wrong layer**: DIM was patched onto the Paragraph container style rather than the Span — text color was unaffected. Fixed to apply `Modifier::DIM` directly to the Span style.
- **Dead `startup_instant` parameter in `border_style`**: breathing border animation was reverted due to `as_rgb()` API issue, but the unused parameter and imports (`Color`, `std::time::Instant`) remained. Removed from signature and all callers.

### Tests
- **101 tests** passing across 14 suites (+3 spinner tests this fix):
  - Spinner: `needs_update` false immediately after update, true after 80ms, frame advances after 80ms
  - Alacritty emulator: bell one-shot consume, `FOCUS_IN_OUT` mode enable/disable via escape sequences
  - ScreenDiff protocol: `focus_events_enabled` propagation, bell cleared by subsequent diff, msgpack roundtrip
  - App logic: `sync_focus_events` gating (4 cases), `from_saved` clamping for corrupted state, tab_scroll reset, empty-tabs safety, mouse-mode cache invalidation on tab switch
  - Protocol serde: `ScreenContent` JSON + msgpack roundtrips preserve `bell` and `focus_events_enabled`; old JSON missing new fields deserializes as `false` (backward compat)
  - Utils: 1000-ID uniqueness under load, hex-alphanumeric format contract

## [0.9.1] - 2026-02-23

### Changed
- README top navigation now uses explicit anchor ids for stable in-page jumps:
  - `Quick Start`, `Features`, `Why Mato?`, `Development`, `Resources`
- README feature descriptions were rewritten in a more developer-friendly and plain-language style.
- Vision table wording was refined (`Pain` + `Mato Solution`) with clearer, shorter pain statements.
- Added Discord community link to README CTA.

### Fixed
- Fixed incorrect/mismatched README jump links caused by emoji heading slug differences.

## [0.9.0] - 2026-02-23

### Added
- **Desk close confirmation**: Sidebar `x` now opens an explicit yes/no confirmation for desk deletion.
- **UI snapshot coverage**: added `tests/ui_snapshot_tests.rs` and snapshot fixtures for key layout states.
- **Release media assets**: added new GIF/APNG image set for docs/website showcases.

### Changed
- **Input semantics (Content focus)**: refined double-ESC handling so Jump Mode entry does not prematurely forward ESC to shell.
- **README and website messaging**: moved to feature-first presentation, improved install guidance for both human and AI-agent flows, and expanded visual showcase coverage.
- **Module organization**:
  - client logic decomposed into dedicated submodules (`jump`, `status`, `mouse`, `ui/*`)
  - daemon provider worker extracted to `src/providers/daemon_provider/worker.rs`
- **Version bump**: `Cargo.toml` updated to `0.9.0`.

### Fixed
- **Bell side effect on `Esc-Esc`**: resolved cases where entering Jump Mode from Content focus could trigger shell bell behavior.
- **Destructive desk close UX risk**: accidental desk deletion is now guarded by explicit user confirmation.

## [0.8.1] - 2026-02-23

### Added
- **Jump labels include digits**: Jump Mode now supports `0-9` labels in addition to letters.
- **ScreenDiff protocol tests**: added coverage for incremental screen update diff computation and application paths.

### Changed
- **Viewport-aware jump targeting**: Jump Mode now allocates labels from currently visible items, not off-screen entries.
  - Sidebar uses current list offset + visible rows.
  - Topbar stays aligned to currently visible tab indices.
- **Keyspace conflict handling in Jump Mode**:
  - Content reserves `c`/`r`/`q` for actions (Copy/Restart/Quit), so they are excluded from jump labels.
  - Sidebar/Topbar reserve `r`/`q` for Rename/Quit.
- **Daemon module naming cleanup**: `src/daemon/daemon.rs` moved to `src/daemon/service.rs`.

### Performance
- **Incremental screen updates (`ScreenDiff`)**: daemon can send changed lines + cursor metadata instead of full-screen payloads for small updates.

### Fixed
- **Jump label mismatch after scroll**: fixed cases where sidebar/topbar labels could point to non-visible targets.
- **Terminal startup/render stability**: initial draw/spawn ordering and non-copy row alignment adjusted to avoid startup offset artifacts.
- **Runtime log noise**: reduced high-frequency info-level connection logs to debug level in daemon/client paths.

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

[Unreleased]: https://github.com/mr-kelly/mato/compare/v0.9.2...HEAD
[0.9.2]: https://github.com/mr-kelly/mato/releases/tag/v0.9.2
[0.9.1]: https://github.com/mr-kelly/mato/releases/tag/v0.9.1
[0.9.0]: https://github.com/mr-kelly/mato/releases/tag/v0.9.0
[0.8.1]: https://github.com/mr-kelly/mato/releases/tag/v0.8.1
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
