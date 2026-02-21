# Release Notes - Mato v0.4.0

**Release Date**: 2026-02-21  
**Status**: Stable

## Overview

Mato v0.4.0 focuses on shell interaction quality, tab responsiveness, and runtime efficiency under heavy multi-tab usage.

## Highlights

- Faster and more stable typing/rendering experience in active terminal tabs
- Better paste semantics (bracketed paste aware)
- Reduced overhead from background polling and screen refresh
- New `--kill` operational command for cleanup/recovery
- Improved status reporting aligned with office/desk/tab model

## What's New

### 1. Shell Input and Paste Experience

- Added dedicated paste path (`ClientMsg::Paste`) from client to daemon.
- Bracketed paste mode is detected and respected, reducing accidental command execution issues in interactive shells/apps.
- Expanded key encoding for terminal content focus (navigation/function keys and modifier combinations).

### 2. Performance and Responsiveness

- Screen fetching now uses a background cached model with adaptive pacing.
- Active tabs are refreshed with higher priority; inactive tabs back off to lower resource usage.
- Reduced render-side overhead by updating cursor style only when it actually changes.
- Fast typing flicker behavior is mitigated by avoiding transient blank-frame fallback patterns.

### 3. Mouse and Interaction Semantics

- Mouse passthrough now depends on app mouse mode state.
- Wheel behavior cleanly falls back to local scrollback when app mouse mode is disabled.

### 4. Operations and Recovery

- Added `mato --kill` to terminate daemon plus related client/tab processes and clean runtime state.
- `mato --status` now reports offices/desks/tabs in the current data model.

### 5. Logging and Noise Reduction

- Expected disconnect scenarios (broken pipe style) are treated as low-severity noise.
- `tab not found` paths used for self-heal are downgraded from alarming log levels.

## Upgrade Notes (from v0.3.0)

- No manual migration required for normal usage.
- Recommended after upgrade:

```bash
mato --kill
mato
```

This ensures daemon/client processes are all on v0.4.0 code paths.

## Quick Validation Checklist

- Type quickly in active tab: no transient terminal blink.
- Paste multi-line content in shell apps: behavior matches bracketed paste expectations.
- Open many tabs and switch rapidly: smoother first-frame response.
- `mato --status`: shows offices/desks/tabs summary.

## Related Docs

- [CHANGELOG.md](../../CHANGELOG.md)
- [RELEASE_SUMMARY_v0.4.0.md](RELEASE_SUMMARY_v0.4.0.md)
- [docs/changelog/](../changelog/)
