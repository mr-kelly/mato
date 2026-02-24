# 2026-02-23 - Rename Input Cursor Editing

## Summary
- Fixed rename popup input editing so users can move the cursor and edit in the middle of text.
- Applied to all rename flows that share the same popup state: desk rename, tab rename, and office rename.

## Changes
- Introduced `RenameState` in `src/client/app.rs`:
  - Stores `target`, `buffer`, and `cursor` (char index).
  - Added editing helpers: `move_left/right/home/end`, `insert_char`, `backspace`, `delete`.
- Updated rename lifecycle in `src/client/app.rs`:
  - `begin_rename_desk` / `begin_rename_tab` now initialize `RenameState`.
  - `commit_rename` now reads from `RenameState.buffer`.
- Updated key handling in `src/client/input.rs` for rename mode:
  - Added support for `Left`, `Right`, `Home`, `End`, `Delete`.
  - `Backspace` and text insert now operate at cursor position.
- Updated popup rendering in `src/client/ui/overlay.rs`:
  - Cursor is rendered at the real editing position (not always at end).
  - When cursor is on a character, that character is highlighted.
  - When cursor is at end, block cursor is shown.

## Tests Updated
- `tests/input_tests.rs`
  - Migrated rename tests to `RenameState`.
  - Added `rename_mode_cursor_and_delete_work_in_middle`.
- `tests/app_tests.rs`
  - Migrated rename state setup to `RenameState`.
- `tests/daemon_tests.rs`
  - Migrated rename state setup and assertions to `RenameState` fields.

## Verification
- Could not run test suite in this environment because `cargo` is not installed (`/bin/bash: cargo: command not found`).

