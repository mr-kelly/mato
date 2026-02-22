# Terminal Rendering Overhaul & Bell Forwarding

**Date**: 2026-02-22
**Session**: Cursor fix, full attribute rendering, bell support, mode detection cleanup

## Overview

Comprehensive terminal rendering overhaul that fixes Claude Code cursor issues, adds complete cell attribute coverage, implements bell forwarding, and simplifies terminal mode detection.

## Changes

### 1. Claude Code Cursor Fix — INVERSE Attribute Rendering

**Problem**: Claude Code's cursor was invisible inside Mato. codex-cli and gemini-cli worked fine.

**Root Cause**: Claude Code sends DECTCEM off (`\x1b[?25l`) to hide the hardware cursor, then renders its own visual cursor using the **INVERSE** (reverse video) text attribute. Mato never captured `Flags::INVERSE` from alacritty_terminal cells, so the visual cursor was invisible.

**Fix**: Added `reverse` field to `ScreenCell`, rendered as `Modifier::REVERSED` in the UI.

### 2. Stale Cursor at Bottom — DECTCEM-Aware Cursor Shape

**Problem**: After fixing the INVERSE cursor, a stale hardware cursor remained at the bottom of the screen.

**Root Cause**: `term.cursor_style().shape` returns the cursor **style preference** (Block/Beam/Underline) but **ignores DECTCEM visibility**. The correct source is `renderable.cursor.shape` from `term.renderable_content()`, which returns `Hidden` when `TermMode::SHOW_CURSOR` is not set.

**Fix**: Switched to `renderable.cursor.shape` for cursor shape detection.

**Key code** (alacritty_terminal-0.25.1, `src/term/mod.rs` ~line 2380):
```rust
let shape = if !vi_mode && !term.mode().contains(TermMode::SHOW_CURSOR) {
    CursorShape::Hidden
} else {
    term.cursor_style().shape
};
```

### 3. Pure Software Cursor Approach

- Hardware cursor is **always hidden** (`terminal.hide_cursor()` at startup)
- Never call `f.set_cursor_position()` — ratatui then always calls `hide_cursor()` in its draw loop
- Software cursor overlay: 1×1 `Paragraph` widget with `REVERSED` modifier at cursor position
- For `Hidden` cursor shape: skip overlay entirely, let inner TUI's own INVERSE-based cursor show through
- Removed all post-draw cursor style propagation code (`SetCursorStyle`, `cursor::Show/Hide`)

### 4. Complete Terminal Cell Attribute Coverage

Added all missing alacritty_terminal `Flags` to `ScreenCell`:

| Flag | ScreenCell Field | Rendered as |
|------|-----------------|-------------|
| `INVERSE` | `reverse` | `Modifier::REVERSED` |
| `DIM` | `dim` | `Modifier::DIM` |
| `STRIKEOUT` | `strikethrough` | `Modifier::CROSSED_OUT` |
| `HIDDEN` | `hidden` | `Modifier::HIDDEN` |
| `DOUBLE_UNDERLINE`, `UNDERCURL`, `DOTTED_UNDERLINE`, `DASHED_UNDERLINE` | `underline` | `Modifier::UNDERLINED` |
| N/A | `underline_color` | SGR 58 underline color via `style.underline_color()` |
| N/A | `zerowidth` | Combining characters appended to base glyph |

Previously captured (unchanged): `BOLD`, `ITALIC`, `UNDERLINE`, `WIDE_CHAR`/spacers.

### 5. Bell (BEL) Forwarding

- Captured `Event::Bell` from alacritty_terminal's `EventListener`
- Added `bell: bool` to `ScreenContent` — piggybacked on existing screen fetch
- Forward `\x07` to host terminal after each draw when bell is triggered
- Works transparently through daemon protocol (serialized with screen data)

### 6. Terminal Mode Detection Simplified

**Before**: Manual byte-scanning of raw escape sequences in `update_terminal_modes()`:
- Scanned for `\x1b[?2004h/l` (bracketed paste)
- Scanned for `\x1b[?1000h/l`, `\x1b[?1002h/l`, `\x1b[?1003h/l` (mouse modes)
- Maintained `mode_tail` buffer for cross-boundary sequence detection
- 53 lines of fragile code + 3 extra struct fields

**After**: Direct TermMode flag queries from alacritty_terminal:
```rust
fn bracketed_paste_enabled(&self) -> bool {
    self.term.mode().contains(TermMode::BRACKETED_PASTE)
}
fn mouse_mode_enabled(&self) -> bool {
    self.term.mode().intersects(TermMode::MOUSE_MODE)
}
```

Eliminated fields: `bracketed_paste`, `mouse_mode`, `mode_tail`.

### 7. ResetTitle Handling

- `Event::ResetTitle` now clears the terminal title (was previously ignored)
- Renamed `TitleCapture` → `EventCapture` listener (handles Title, ResetTitle, Bell)

### 8. Debug Logging Cleanup

Removed all diagnostic logging added during cursor debugging:
- `cursor-debug` tracing (tab name, cursor position, shape, visible area)
- `bottom-cell-debug` tracing
- `last_cursor_debug_log` field from App
- `last_cursor_shape` field from App
- `current_cursor_shape()` method from App

## Files Changed

| File | Changes |
|------|---------|
| `src/emulators/alacritty_emulator.rs` | EventCapture (Bell/ResetTitle), TermMode flags, cursor shape source fix, all cell flags |
| `src/terminal_provider.rs` | New ScreenCell fields (dim, reverse, strikethrough, hidden, underline_color, zerowidth), bell in ScreenContent |
| `src/client/ui.rs` | Render all new attributes, bell flag, removed debug logging, removed `set_cursor_position` |
| `src/emulators/vt100_emulator.rs` | New ScreenCell fields (reverse from `cell.inverse()`), bell field |
| `src/main.rs` | `hide_cursor()` at startup, bell forwarding, removed cursor style propagation block |
| `src/client/app.rs` | Removed `last_cursor_shape`/`last_cursor_debug_log`/`current_cursor_shape()`, added `pending_bell` |

## Stats

- Build: zero warnings
- Tests: all passing (99 tests + 4 ignored)
- Net line change: significant reduction from removed byte-scanning + debug code

## Follow-up Fixes (Same Session)

### 9. Onboarding Screen Residual Content (New Office)

**Problem**: Clicking `New Office` could show old content at the bottom, overlapping onboarding UI.

**Fix**: In onboarding render path, force a full-frame clear before drawing widgets:

- Added `Clear` widget import in `src/client/onboarding_tui.rs`
- Added `f.render_widget(Clear, f.area())` at top of `draw_onboarding()`

This prevents stale cells from previous TUI frames from remaining visible.

### 10. Root Fix: Single Terminal Owner During Runtime Onboarding

**Problem**: Runtime `New Office` onboarding previously switched terminal modes twice (main UI and onboarding both managed alt-screen/raw-mode), causing fragile transitions and occasional render residue.

**Fix**:

- Added `show_onboarding_in_terminal(&mut Terminal<...>)` in `src/client/onboarding_tui.rs`
- Main client loop now calls this function directly and stays in the same terminal session
- Removed runtime onboarding path's `disable_raw_mode + LeaveAlternateScreen` and re-enter sequence from `src/main.rs`

**Result**: During normal app runtime, terminal mode ownership is single-source (main client), and onboarding only renders UI logic.

### 11. Main Loop ScreenState Refactor (No Nested Onboarding Loop)

**Problem**: Runtime onboarding was still running its own blocking event loop, which made terminal ownership and redraw semantics harder to reason about.

**Fix**:

- Added `ScreenState` in `src/main.rs`:
  - `Main`
  - `Onboarding(OnboardingController)`
- Added `OnboardingController` in `src/client/onboarding_tui.rs` with:
  - `draw(&mut Frame)`
  - `handle_key(KeyEvent) -> OnboardingAction`
- Main loop now routes draw/input by current screen state in one unified event loop.

**Result**: Main UI and onboarding now share one render/input loop, reducing mode-transition fragility and eliminating nested TUI control flow during runtime onboarding.

### 12. Onboarding Cancel Policy Split (First Run vs New Office)

**Problem**: First-run onboarding (`state.json` missing) could be canceled via `Esc/q`, which left users in a broken startup path.

**Fix**:

- Added cancel policy to `OnboardingController`:
  - `new_required()` for first-run onboarding (no global cancel)
  - `new_optional()` for in-app `New Office` onboarding (supports cancel)
- First-run path now requires completion and returns `SavedState` directly.
- Help text is now mode-aware:
  - Required mode: no `Esc/q Cancel` hint
  - Optional mode: retains `Esc/q Cancel` hint

**Result**:
- Fresh install onboarding cannot be exited with `Esc/q`; user must select a template and continue.
- `New Office` onboarding still allows `Esc/q` back to the main UI.

### 13. Policy Adjustment: First-Run Supports `q` Quit, In-App Uses `Esc` Back

Based on interactive validation feedback, onboarding key policy was finalized as:

- **First run** (no `state.json`): `q` quits setup flow and exits cleanly.
- **In-app New Office**: `Esc` returns to main UI.

Help text was updated to match actual mode behavior (`q Quit` vs `Esc Back`), and runtime onboarding mode wiring in `main.rs` now instantiates explicit in-app mode.

### 14. Startup Onboarding Residue Cleanup

Added immediate startup onboarding clear before first frame after entering alternate screen:

- `EnterAlternateScreen`
- terminal clear
- render onboarding

This removes shell ghosting on first-run onboarding startup.

### 15. Release Prep (v0.7.1)

Low-risk cleanup and release preparation updates:

- Simplified nested style conditionals in `src/client/ui.rs`
- Replaced manual parity check with `.is_multiple_of(2)`
- Added `Default` implementation for `TerminalGuard`
- Removed unnecessary cast in alacritty emulator
- Prepared `CHANGELOG.md` and `docs/release-notes/*v0.7.1*`
- Reordered onboarding templates to prioritize:
  - `Start from Scratch` at top
  - `Mato Creator Office` in second position
- Expanded `Start from Scratch` template to 3 desks with 2 tabs each.

### 16. Terminal Runtime Capability Audit + Roadmap Draft

Created a full capability checklist and gap analysis for coding-agent execution:

- Added `docs/todos/roadmap.md` with sectioned audit across:
  - PTY/process lifecycle
  - input system
  - ANSI/render pipeline
  - multiplexer/session model
  - task/agent orchestration
  - plugin extensibility
  - persistence/config/security/testing
- Marked each capability as implemented/partial/missing and listed prioritized execution phases (P0-P3), with shell/terminal core UX as top priority.
- Updated `docs/todos/README.md` index to include the new roadmap.
