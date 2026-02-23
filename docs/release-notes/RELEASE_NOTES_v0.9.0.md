# Release Notes - Mato v0.9.0

**Release Date**: 2026-02-23  
**Status**: Stable

## Overview

v0.9.0 is a UX and maintainability release focused on safer desk operations, cleaner input behavior, stronger documentation/website onboarding, and internal refactoring for better code structure.

## Highlights

- Desk close in Sidebar now requires explicit confirmation (`y`/`Enter` or `n`/`Esc`).
- Tab close in Topbar remains immediate (no confirmation), preserving fast tab workflow.
- Fixed `Esc-Esc` in Content focus to avoid accidental bell-triggering side effects.
- README and website install flow were redesigned for faster onboarding (human + AI agent paths).
- Internal modules were split/refactored (client UI decomposition and daemon worker extraction) with snapshot tests added.

## What's New

### 1. Safer Desk Deletion UX

- Pressing `x` on a Desk now opens a yes/no confirmation dialog.
- Confirmation clearly states desk-wide impact (all tabs / PTYs in that desk).
- Keyboard flow:
  - `y` or `Enter`: confirm deletion
  - `n` or `Esc`: cancel

### 2. Input Behavior Fix: Esc Double-Press

Content-focus `Esc-Esc` now enters Jump Mode without forwarding an early standalone `Esc` into the shell first, preventing bell side effects in some terminal apps.

Single `Esc` behavior is preserved via delayed forwarding.

### 3. Docs + Website Refresh

- README now follows a feature-first structure with richer media.
- Added/updated APNG and GIF-based feature previews (Jump Mode, themes, onboarding, multi-client sync, spinner activity, persistence).
- Installation guidance is clearer for both manual users and AI coding-agent users.

### 4. Internal Refactor + Test Coverage

- Client internals were split into clearer modules (jump/status/mouse/ui submodules).
- Daemon provider worker logic was extracted into a dedicated module.
- Added UI snapshot tests and snapshots to lock rendering behavior.

## Upgrade Notes

Recommended restart flow:

```bash
mato --kill
mato
```

## Related Docs

- [CHANGELOG.md](../../CHANGELOG.md)
- [RELEASE_SUMMARY_v0.9.0.md](RELEASE_SUMMARY_v0.9.0.md)
