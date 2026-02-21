# 2026-02-22 Switch to alacritty_terminal Emulator

## Summary

Replaced the incomplete `VteEmulator` with a full-featured `AlacrittyEmulator` backed by the `alacritty_terminal` crate. `vt100` remains as an explicit fallback.

## Changes

### New: `AlacrittyEmulator` (`src/emulators/alacritty_emulator.rs`)

- Uses `alacritty_terminal::Term` — the same emulator powering Alacritty
- Full VT/xterm support: scrolling, colors (RGB + 256 + named), bold/italic/underline, alternate screen, wide chars
- Implements `TerminalEmulator` trait: `process`, `get_screen`, `resize`
- `NoopEventListener` handles title/bell events silently

### Updated: `src/providers/pty_provider.rs`

- Default emulator: `alacritty` (any unknown value also uses alacritty)
- Fallback: `vt100` (set `emulator = "vt100"` in config)
- Removed `vte` option

### Updated: `src/config.rs`

- `default_emulator()` returns `"alacritty"`

### Updated: `src/emulators/mod.rs`

- Added `alacritty_emulator` module and `AlacrittyEmulator` export

### Updated: `Cargo.toml`

- Added `alacritty_terminal = "0.25"`

## Motivation

The previous `VteEmulator` was a skeleton — `csi_dispatch` was empty, meaning no colors, no cursor movement, and no scrolling. Content would pile up in the top-left corner and stop scrolling at the bottom.

`alacritty_terminal` provides a battle-tested, complete terminal emulator with zero additional implementation burden.
