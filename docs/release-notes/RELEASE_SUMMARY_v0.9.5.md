# Mato v0.9.5 Release Summary

## Theme

Documentation structure and contributor workflow clarity.

## Major Changes

### README

- Removed inline test-suite matrix/details from README.
- Replaced with direct link to dedicated testing docs.

### Testing Docs

- Added `docs/TESTING.md`:
  - full test command
  - snapshot workflow
  - environment caveat note

### TODOs / Roadmap

- Refreshed `docs/todos/roadmap.md` capability status markers and baseline version.
- Replaced legacy `docs/todos/TODO.md` with concise execution backlog synced to roadmap priorities.
- Updated `docs/todos/README.md` to match new document roles.

### Changelog Session Record

- Added `docs/changelog/2026-02-24_todos-roadmap-audit.md` for this doc alignment session.

## Verification

```bash
source ~/.cargo/env && COLORTERM= cargo test
```
