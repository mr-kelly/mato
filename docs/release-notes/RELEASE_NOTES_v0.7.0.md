# Release Notes - Mato v0.7.0

**Release Date**: 2026-02-22  
**Status**: Stable

## Overview

v0.7.0 is a terminal rendering architecture release focused on correctness for modern interactive CLIs. It improves cursor semantics, expands terminal attribute fidelity, and introduces bell forwarding while simplifying internal mode detection.

## Highlights

- Claude Code cursor visibility issue addressed via full INVERSE attribute propagation.
- Rendering pipeline now carries richer cell attributes end-to-end.
- Bell (`BEL`) events are forwarded to the host terminal.
- Cursor handling aligned with DECTCEM hidden/show semantics.
- Terminal mode detection moved from manual byte scanning to native `TermMode` flags.

## What's New

### 1. Rendering Fidelity Upgrade

`ScreenCell` now includes additional rendering metadata used by UI:

- `display_width`
- `dim`
- `reverse`
- `strikethrough`
- `hidden`
- underline color and zero-width combining support

This improves compatibility with rich TUI apps that rely on nuanced SGR styling.

### 2. Cursor Pipeline Rework

- Cursor shape now follows renderable cursor state (including `Hidden`) rather than style preference alone.
- Software cursor overlay flow was cleaned up to reduce stale cursor artifacts.
- Debug telemetry was added during investigation to capture shape/row/col/viewport data for problematic tabs.

### 3. Bell Forwarding

- Emulator now captures `Bell` events and propagates them through `ScreenContent`.
- Client forwards bell to host terminal after draw when triggered.

### 4. Mode Detection Simplification

Removed fragile escape-sequence byte scanning and state tail buffers for:

- bracketed paste
- mouse mode

Now derived directly from `alacritty_terminal::TermMode`.

### 5. Title Reset Correctness

- `ResetTitle` is now handled explicitly, preventing stale terminal title display.

## Upgrade Notes

Recommended restart flow after upgrade:

```bash
mato --kill
mato
```

## Related Docs

- [CHANGELOG.md](../../CHANGELOG.md)
- [RELEASE_SUMMARY_v0.7.0.md](RELEASE_SUMMARY_v0.7.0.md)
- [2026-02-22_terminal-rendering-overhaul-and-bell-forwarding.md](../changelog/2026-02-22_terminal-rendering-overhaul-and-bell-forwarding.md)
