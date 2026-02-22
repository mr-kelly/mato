# Release Notes - Mato v0.5.0

**Release Date**: 2026-02-22  
**Status**: Stable

## Overview

v0.5.0 focuses on interaction consistency and high-scale usability: smoother shell rendering, stronger system-theme readability, and a much more capable Jump Mode for large workspaces.

## Highlights

- Jump Mode now supports `a-z/A-Z` (52 labels)
- Better Jump behavior in large templates (balanced desk/tab target allocation)
- Directional focus switching matrix is now explicit and consistent
- Onboarding is faster: default office name + one-key template start
- "Power User" template renamed to **Mato Creator Office**

## What's New

### 1. Jump Mode for Real Large Workspaces

- Labels expanded from 26 to 52 (`a-z/A-Z`).
- Target allocation strategy by focus:
  - **Content**: balanced interleaving of tabs and desks
  - **Topbar**: tabs first
  - **Sidebar**: desks first
- Rendering and key mapping now share one unified target model.

### 2. Focus Switching Rules in Jump Mode

Directional arrows now follow a fixed matrix:

- **Topbar**: `←` Sidebar, `↓` Content
- **Sidebar**: `→` Content, `↑` Topbar
- **Content**: `←` Sidebar, `↑` Topbar

`w/a` are no longer treated as focus shortcuts, so letter jumps stay predictable.

### 3. Onboarding Flow Upgrade

- Default office name is prefilled from local identity (username/hostname), with length-safe formatting.
- Users can immediately choose template and start (`↑↓ + Enter`).
- Optional rename mode is available via `r` before applying template.

### 4. Template Rename

- **Power User** -> **Mato Creator Office**
- Updated wording to describe the creator/builder workflow explicitly.

### 5. UI and Theme Polish

- System theme readability improved for focus and selection states.
- Jump popup rendering no longer visually bleeds through background text.
- Sidebar jump labels align correctly with desk rows.
- Top-right daemon status indicator semantics clarified:
  - connected: `✓`
  - connecting/in progress: `·` / `•`

## Upgrade Notes

Recommended restart flow after upgrade:

```bash
mato --kill
mato
```

This ensures both daemon and clients are on v0.5.0 behavior.

## Related Docs

- [CHANGELOG.md](../../CHANGELOG.md)
- [RELEASE_SUMMARY_v0.5.0.md](RELEASE_SUMMARY_v0.5.0.md)
- [2026-02-22_shell-experience-jump-mode-and-system-theme-polish.md](../changelog/2026-02-22_shell-experience-jump-mode-and-system-theme-polish.md)
