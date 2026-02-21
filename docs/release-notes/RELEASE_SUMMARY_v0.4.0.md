# Mato v0.4.0 Release Summary

## Theme

v0.4.0 is a shell experience and runtime-efficiency release focused on perceived smoothness and practical operator controls.

## Major Technical Changes

### Input Pipeline

- Added protocol-level paste message (`ClientMsg::Paste`) and daemon handling.
- `Event::Paste` now routes through a dedicated paste path instead of raw byte write.
- Bracketed paste mode detection implemented in emulator path and exposed through provider capabilities.

### Rendering and Flicker Control

- Active typing flicker mitigated by preserving previous screen content during transient fetch failures.
- Cursor style control sequence emission changed from per-frame to on-change only.

### Screen Fetch Architecture

- Daemon provider now uses background worker + cache for screen content.
- Adaptive polling strategy:
  - recent demand: fast refresh
  - inactive demand: slower refresh
  - long inactive period: worker exits
- Worker-side fetch now supports connection reuse strategy to reduce connect/disconnect churn.

### Mouse/Input Mode Semantics

- Introduced `GetInputModes` / `InputModes` protocol for querying mouse/bracketed-paste mode.
- Mouse passthrough policy now aligns with app mouse mode state.

### Runtime/Operations

- Added `mato --kill` command path for daemon + client/tab process cleanup.
- `--status` parsing/output aligned to current `SavedState` schema (offices/desks/tabs).

### Logging Hygiene

- Reduced noise for expected disconnect behavior (`Broken pipe` class).
- Lowered severity for self-heal `tab not found` paths.

## User-Facing Impact

- Faster tab switching perception under load.
- Less visual disturbance while typing quickly in active shell tabs.
- Better paste safety/compatibility in interactive terminal programs.
- Clearer operational controls (`--kill`, updated `--status`).

## Verification

- Local verification command:

```bash
source ~/.cargo/env && cargo test -q
```

- Result: test suite passed.

## Release Artifacts

- `CHANGELOG.md` updated with v0.4.0 section.
- `docs/release-notes/RELEASE_NOTES_v0.4.0.md` added.
- `docs/release-notes/RELEASE_SUMMARY_v0.4.0.md` added.
- `Cargo.toml` version bumped to `0.4.0`.
