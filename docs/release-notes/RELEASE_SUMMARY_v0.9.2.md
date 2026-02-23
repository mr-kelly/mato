# Mato v0.9.2 Release Summary

## Theme

Correctness and signal-quality release: focus-event protocol gating, bell consume semantics, spinner redraw stability, and UX polish through toast + jump dim overlays.

## Major Changes

### Runtime Fixes

- Focus event emission now checks `focus_events_enabled` before sending `FocusIn`/`FocusOut`.
- Screen bell cache is consumed once to avoid repeated bell playback.
- `focus_events_enabled` now participates in daemon ScreenDiff metadata change detection.
- Spinner redraw path now includes timer-driven updates while active tabs exist.

### UX Improvements

- Toast notifications for create/close/rename operations.
- Jump Mode now dims terminal background for clearer label targeting.
- Cleaned stale style/parameter remnants around border styling paths.

### Website

- GA4 integrated with Next.js official third-party component package.

## Tests

- Full suite passes, including updated coverage for:
  - focus-event gating
  - bell consume behavior
  - ScreenDiff metadata propagation
  - spinner timing behavior
  - protocol backward compatibility and id generation guarantees

## Verification

```bash
source ~/.cargo/env && cargo test
pnpm -C website types:check
```
