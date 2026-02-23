# Release Notes - Mato v0.8.1

**Release Date**: 2026-02-23  
**Status**: Stable

## Overview

v0.8.1 is a focused UX and stability release on top of v0.8.0. It improves Jump Mode correctness in scrolled/partial views, extends jump labels with digits, and tightens startup/render behavior while reducing noisy runtime logs.

## Highlights

- Jump labels now match what is actually visible in Sidebar/Topbar.
- Jump Mode supports number keys (`0-9`) in label space.
- Content Jump Mode keeps `c`/`r`/`q` for actions and excludes them from jump target labels.
- Incremental `ScreenDiff` paths and related tests are now part of this release line.
- Startup/render behavior and daemon logging noise were refined for cleaner operation.

## What's New

### 1. Jump Mode Viewport Correctness

Jump target assignment now follows the current visible viewport:

- Sidebar labels come from current list offset + visible rows.
- Topbar labels stay tied to visible tab indices.

This prevents off-screen target assignment and label/target mismatch after scrolling.

### 2. Jump Label Keyspace Improvements

- Label pool now includes digits (`0-9`) in addition to letters.
- Reserved action keys are excluded from jump label assignment by focus:
  - Content: `c`, `r`, `q`
  - Sidebar/Topbar: `r`, `q`

### 3. Incremental Screen Update Path

`ScreenDiff` incremental updates are integrated in the release line with test coverage, reducing unnecessary full-frame transfer/render for small terminal changes.

### 4. Stability and Noise Reduction

- Startup/render sequence was adjusted to reduce transient first-frame alignment artifacts.
- High-frequency connection lifecycle logs were moved from `info` to `debug` in daemon/client flows.

## Upgrade Notes

Recommended restart flow:

```bash
mato --kill
mato
```

## Related Docs

- [CHANGELOG.md](../../CHANGELOG.md)
- [RELEASE_SUMMARY_v0.8.1.md](RELEASE_SUMMARY_v0.8.1.md)
