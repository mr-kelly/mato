# 2026-02-23 - Terminal Multi-Client & Core Bug Fixes

## Session Goal

Continue fixing terminal core experience bugs discovered during multi-client (phone mosh + PC mosh)
usage. Complete the in-progress `alacritty_emulator` bottom-rows fix, then audit and fix adjacent
bugs in the same class.

---

## Bugs Fixed

### 1. Multi-client screen sync — alacritty emulator shows top rows instead of bottom (build fix)

The previous session left a type error in `src/emulators/alacritty_emulator.rs` line 256:
`cur.column.0.max(0).min(render_cols as i32 - 1)` — `column.0` is `usize`, not `i32`.

**Fix**: changed to `.min(render_cols.saturating_sub(1))` (pure `usize` arithmetic).

Also fixed a logic ordering issue: `(abs_line - visible_top_line) as usize` could silently wrap to
a huge value when `abs_line - visible_top_line < 0` (negative row in history). Refactored to check
`row_i32 < 0` before the cast:

```rust
let row_i32 = abs_line - visible_top_line;
if row_i32 < 0 || col >= render_cols {
    continue;
}
let row = row_i32 as usize;
```

**Files**: `src/emulators/alacritty_emulator.rs`

---

### 2. vt100 emulator — same top-rows bug as alacritty

`Vt100Emulator::get_screen` iterated `for row in 0..rows` calling `screen.cell(row, col)`, which
always accesses the **top** `rows` rows of the PTY. For a smaller subscriber, the shell/cursor lives
at the bottom and would be invisible.

**Fix**: same row_offset logic as the alacritty fix:

```rust
let (screen_rows, screen_cols) = screen.size();
let render_rows = rows.min(screen_rows);
let render_cols = cols.min(screen_cols);
let row_offset = screen_rows - render_rows;

for display_row in 0..render_rows {
    let pty_row = row_offset + display_row;
    // use screen.cell(pty_row, col)
}

// Cursor adjustment:
let display_cr = if cr >= row_offset { (cr - row_offset).min(render_rows - 1) } else { 0 };
```

**Files**: `src/emulators/vt100_emulator.rs`

---

### 3. Multiple clients fight over PTY size (resize storm)

**Root cause**: Default `resize_strategy = Sync`. Each client's window resize (e.g., phone keyboard
popup) sent `ClientMsg::Resize` to the daemon, which unconditionally called `tab.resize(r, c)`.
With two clients of different sizes the PTY oscillated between their sizes, sending repeated
SIGWINCH to the shell. TUI apps (vim, htop, lazygit) on both clients reflowed constantly.

**Fix**: Added `subscriber_count: Arc<AtomicUsize>` to `PtyProvider`. The daemon push loop:
- Increments `subscriber_count` when a `Subscribe` message is received and the push loop starts
- Decrements it when the push loop exits (client disconnects)
- Only calls `tab.resize(r, c)` when `subscriber_count <= 1`

With multiple subscribers, `sub_rows`/`sub_cols` are still updated so `get_screen` returns the
correct bottom-rows viewport for each client's size — but the PTY itself stays at its original
(first-subscriber) size.

**Files**: `src/providers/pty_provider.rs`, `src/daemon/service.rs`

---

## Tests Updated

- `tests/vt100_emulator_tests.rs`: updated `vt100_get_screen_smaller_than_parser_returns_correct_size`
  to write content at the **bottom** of the PTY (`\x1b[24;1H`) and assert it appears on the last
  display row (display row 4 of a 5-row window into a 24-row terminal). Also asserts cursor row = 4.

---

## Architectural Notes (Known Limitations)

The following multi-client issues remain and require larger refactors:

| Issue | Impact | Fix approach |
|-------|--------|--------------|
| **Shared scroll state** | Client A copy-mode scroll → Client B also sees scrolled view | Per-subscriber `scroll_offset` in push loop + `get_screen_scrolled(rows, cols, offset)` API |
| **Mouse coordinates not adjusted for row_offset** | Click at row 5 of phone → PTY receives row 5, but shell is at row 20 → vim/ranger click positions wrong | Daemon intercepts mouse escape sequences per-subscriber and adjusts row by `row_offset` |

The single-client case is unaffected by both. Multi-client is the only scenario that triggers these.

---

## Verification

```
cargo test  →  264 tests, 0 failed
cargo build →  0 errors
```
