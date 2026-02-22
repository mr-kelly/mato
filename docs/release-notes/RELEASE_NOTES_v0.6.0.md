# Release Notes - Mato v0.6.0

**Release Date**: 2026-02-22  
**Status**: Stable

## Overview

v0.6.0 focuses on reliability in real interactive workflows: dead tabs now recover automatically, desk switching is more proactive, and terminal rendering has better cursor stability for rich CLIs. This release also ships a full website refresh aligned with Mato's core positioning.

## Highlights

- Exited tab shells auto-respawn instead of staying stuck.
- Desk switching now proactively starts the target active tab.
- Missing-tab first frame recovers faster with synchronous spawn + retry.
- Cursor/render stability improved for wide-character spacer handling.
- Website redesigned and aligned with README narrative.

## What's New

### 1. Terminal Lifecycle Resilience

- Added PTY child liveness checks and auto-respawn behavior.
- Daemon paths now ensure PTY is running before handling:
  - input
  - paste
  - input mode query
  - screen fetch
  - scroll

This addresses cases where a shell `exit` left a tab unusable.

### 2. Faster Desk/Tab Recovery

- Sidebar desk navigation now proactively spawns the target desk's active tab.
- Jump-mode desk selection also proactively spawns the destination tab.
- On `tab not found`, client now does synchronous `Spawn` and immediate `GetScreen` retry once.

This reduces blank/empty terminal windows during desk switches.

### 3. Cursor and Rendering Stability

- `sync_tab_titles()` now uses current terminal dimensions (instead of `1x1` fetches).
- Alacritty emulator output handling now treats wide-char spacer cells as zero-width placeholders.

This lowers cursor misalignment risk in richer CLIs.

### 4. Website Refresh

- Home page restructured around product value and workflows.
- Added showcase, problem/solution comparisons, feature pillars, and shortcut philosophy section.
- Build-time website asset sync now includes:
  - `logo.svg`
  - screenshot image
- `/website` scripts standardized on `pnpm` flow.

## Upgrade Notes

Recommended restart flow after upgrade:

```bash
mato --kill
mato
```

This ensures daemon and clients are running with v0.6.0 behavior.

## Related Docs

- [CHANGELOG.md](../../CHANGELOG.md)
- [RELEASE_SUMMARY_v0.6.0.md](RELEASE_SUMMARY_v0.6.0.md)
- [2026-02-22_onboarding-office-name-simplification.md](../changelog/2026-02-22_onboarding-office-name-simplification.md)
