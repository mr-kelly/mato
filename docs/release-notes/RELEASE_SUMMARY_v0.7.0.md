# Mato v0.7.0 Release Summary

## Theme

Terminal rendering correctness and cursor semantics for modern full-screen CLI apps.

## Major Changes

### Rendering Core

- Alacritty extraction path moved to renderable display iteration.
- Expanded `ScreenCell` model to preserve more terminal attributes and width semantics.
- UI renderer updated to consume new style fields (`reverse`, `dim`, `hidden`, etc.).

### Cursor Semantics

- Cursor shape handling aligned with DECTCEM visibility (`Hidden` included).
- Removed stale cursor side effects from previous mixed hardware/software handling paths.
- Added debug instrumentation used to analyze cross-CLI behavior differences.

### Bell and Events

- Added bell event capture in emulator event listener.
- Added `bell` flag propagation in `ScreenContent`.
- Client now forwards bell after frame draw.

### Terminal Mode Handling

- Replaced manual mode byte scanning logic with direct `TermMode` checks.
- Removed auxiliary sequence-tail buffering and related transient state complexity.

### Adapter Consistency

- `vt100` adapter updated to populate newly added `ScreenCell` fields for parity with alacritty-backed path.

## Verification

Validation command:

```bash
source ~/.cargo/env && cargo test -q
```

Result: passing in this release prep session.
