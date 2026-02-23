# Release Notes - Mato v0.9.2

**Release Date**: 2026-02-23  
**Status**: Stable

## Overview

v0.9.2 focuses on runtime correctness and UX polish: focus-event reliability, bell handling correctness, spinner smoothness under idle input, and stronger visual/interaction feedback via toast and jump-background dimming.

## Highlights

- Fixed literal `^[[I` / `^[[O` artifacts during tab switch by gating focus events to PTYs that opted in.
- Fixed repeated bell ringing from cached screen state by consuming bell once per read.
- Spinner now continues animating even without keyboard input when tabs are active.
- Added toast notifications for key actions (create/close/rename flows).
- Added Jump Mode background dim for clearer target labels.
- Added GA4 integration on the website via Next.js `@next/third-parties`.

## What's New

### 1. Toast Notification System

- New transient toast state (`3s` visibility) with fade behavior.
- Triggered for high-signal actions:
  - desk created
  - desk closed (confirmed)
  - desk/tab/office renamed

### 2. Jump Mode Visual Clarity

- Entering Jump Mode now dims background terminal content, making jump labels easier to scan.

### 3. Focus-Event Correctness

- Focus in/out sequences are now sent only when the active PTY explicitly enables focus tracking (`\x1b[?1004h`).
- Prevents raw escape sequence leakage into shells/apps that did not opt in.

### 4. Bell Handling Fix

- Bell in screen cache is now consume-on-read, preventing repeated bell playback across frames.

### 5. Spinner Redraw Reliability

- Spinner updates now trigger redraw timing independently of user input while active tabs exist.

### 6. Website Analytics

- Added Google Analytics 4 integration using Next.js official third-party helper:
  - package: `@next/third-parties`
  - ID: `G-Q30J2ZFNE4`

## Validation

```bash
source ~/.cargo/env && cargo test
pnpm -C website types:check
```

## Related Docs

- [CHANGELOG.md](../../CHANGELOG.md)
- [RELEASE_SUMMARY_v0.9.2.md](RELEASE_SUMMARY_v0.9.2.md)
