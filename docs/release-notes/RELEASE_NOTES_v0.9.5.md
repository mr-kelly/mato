# Release Notes - Mato v0.9.5

**Release Date**: 2026-02-24  
**Status**: Stable

## Overview

v0.9.5 is a documentation and contributor-experience release. This version streamlines where users and contributors find testing, roadmap, and execution-backlog information.

## Highlights

- README is cleaner: detailed test-suite tables were removed from the homepage flow.
- Added dedicated testing guide: `docs/TESTING.md`.
- TODO system aligned:
  - `docs/todos/roadmap.md` refreshed to v0.9.5 baseline and current capability state.
  - `docs/todos/TODO.md` rewritten as a concise execution backlog (P0-P4).
  - `docs/todos/README.md` updated to reflect the new source-of-truth structure.
- Added session changelog record for this doc alignment pass.

## What's New

### 1. Dedicated Testing Document

Testing instructions are now centralized in `docs/TESTING.md`:

- run full Rust tests
- run/update snapshot tests
- basic environment notes

### 2. Roadmap and TODO Realignment

`roadmap.md` now reflects current implementation reality (v0.9.5), including:

- adaptive polling + conditional render status
- partial `cwd` support status
- resize strategy current state (`fixed`/`sync`)
- graphics work moved from initial implementation framing to compatibility hardening

`TODO.md` is now execution-first and synced with roadmap priorities.

### 3. README Content Focus

The README now links to testing docs instead of embedding long test-suite breakdowns, improving readability for new users.

## Validation

```bash
source ~/.cargo/env && COLORTERM= cargo test
```

## Related Docs

- [CHANGELOG.md](../../CHANGELOG.md)
- [RELEASE_SUMMARY_v0.9.5.md](RELEASE_SUMMARY_v0.9.5.md)
- [docs/TESTING.md](../TESTING.md)
