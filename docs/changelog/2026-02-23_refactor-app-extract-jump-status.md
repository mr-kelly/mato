# 2026-02-23: Refactor app.rs — Extract jump and status modules

## Summary

Extracted two groups of methods from `src/client/app.rs` (1021 lines) into
dedicated files to improve separation of concerns.

## Changes

### `src/client/jump.rs` (new)
Moved from `impl App` in `app.rs`:
- `fn jump_key_reserved_for_focus` (now `pub(super)`)
- `pub fn jump_labels`
- `fn visible_desk_indices` (now `pub(super)`)
- `pub fn jump_targets`

### `src/client/status.rs` (new)
Moved from `impl App` in `app.rs`:
- `pub fn sync_tab_titles`
- `pub fn refresh_active_status`
- `fn spawn_active_status_worker`
- `pub fn refresh_update_status`
- `pub fn refresh_update_status_from_socket`
- `pub fn update_spinner`
- `pub fn get_spinner`

### `src/client/app.rs`
- Removed the 9 methods listed above
- Cleaned up imports that were only needed by moved methods
  (`ClientMsg`, `ServerMsg`, `BufRead`, `BufReader`, `Write`, `UnixStream`,
  `Path`, `mpsc::self`, `thread`)
- Made `active_status_rx` field `pub(crate)` so `status.rs` can access it

### `src/client/mod.rs`
- Added `pub mod jump;`
- Added `pub mod status;`

## Result

`cargo build` → zero errors, zero warnings (in this crate).
`app.rs` reduced from 1021 → ~840 lines.
