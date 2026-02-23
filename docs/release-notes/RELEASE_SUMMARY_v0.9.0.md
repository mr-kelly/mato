# Mato v0.9.0 Release Summary

## Theme

Safer desk lifecycle actions, cleaner ESC interaction semantics, and a major docs/website polish with incremental internal refactoring.

## Major Changes

### Desk Close Confirmation

- Added desk-level delete confirmation state and popup flow.
- Sidebar `x` now routes through explicit confirm/cancel handling.
- Topbar tab close behavior intentionally remains direct/no-confirm.

### ESC Double-Press Reliability

- Updated Content-focus ESC handling:
  - double-ESC enters Jump Mode without premature ESC passthrough
  - single ESC remains supported through deferred flush behavior
- Eliminates bell side effects seen in some shell applications on `Esc-Esc`.

### Docs / README / Website Refresh

- README:
  - updated section structure (features-first)
  - improved contributor guidance (including coding-agent flow)
  - richer showcase media and consistent visual formatting
- Website:
  - install UX refinement (AI-agent/human pathways)
  - layout and spacing improvements
  - dynamic version badge component support

### Refactor and Module Decomposition

- Client-side module extraction:
  - jump/status/mouse separation
  - UI split to `ui/mod.rs` + focused submodules
- Daemon provider worker extraction:
  - `src/providers/daemon_provider/worker.rs`

### Tests

- Added UI snapshot test suite and snapshot artifacts.
- Existing unit/integration coverage kept green after refactor and UX fixes.

## Verification

Validation commands:

```bash
source ~/.cargo/env && cargo build
source ~/.cargo/env && cargo test
```
