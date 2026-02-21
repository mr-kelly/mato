# 2026-02-21 TODO Audit, Clippy Cleanup, and Tab Title Separation

## Overview

This update focused on documentation hygiene, warning reduction, and a UI naming behavior fix:

1. Audited `docs/todos/TODO.md` and marked historically completed items.
2. Reduced compiler/clippy noise with low-risk cleanup changes.
3. Separated tab config name from runtime terminal title in UI rendering.
4. Simplified Office selector label text.

---

## 1) TODO Audit and Backfill

`docs/todos/TODO.md` was reviewed against current repository state and historical release docs.

### What was updated
- Marked clearly completed historical items as `[x]` (release checklist, completed phase items, existing docs/tests).
- Synced header version to `0.3.0`.
- Kept genuinely incomplete roadmap/future items as `[ ]`.

### Why
- The TODO file had mixed historical status and future roadmap entries, making completion status unclear.

---

## 2) Warning and Clippy Cleanup

A low-risk cleanup pass was applied to remove common warnings without changing feature behavior.

### Changes made
- Removed unused imports/variables and dead helper code.
- Handled `save_state` results explicitly with warning logs.
- Fixed function-pointer cast warning in `SIGCONT` registration.
- Applied clippy suggestions:
  - `int_plus_one`
  - `collapsible_if`
  - `field_reassign_with_default`
  - `new_without_default` (added `Default` impls where appropriate)
  - `needless_range_loop`
  - `io_other_error`

### Remaining clippy warning
- `clippy::module_inception` at `src/daemon/mod.rs` (`pub mod daemon;`).
- This is a structural naming issue and was intentionally left unchanged in this cleanup pass.

---

## 3) Tab Name vs Terminal Title (UI Behavior Fix)

Previous behavior: active terminal title could overwrite `tab.name`, causing topbar labels to drift from configured names.

### New behavior
- **Topbar**: shows tab config name only (`tab.name`).
- **Content title**: shows `tab_config_name : terminal_title` when runtime terminal title exists.

### Implementation
- Added runtime map in app state:
  - `terminal_titles: HashMap<String, String>` keyed by `tab_id`.
- Updated `sync_tab_titles()`:
  - no longer mutates `tab.name`
  - stores title into `terminal_titles`
- Updated terminal pane title rendering:
  - uses `tab.name` by default
  - appends runtime title when present

### Rationale
- Preserve stable user-configured tab names for navigation.
- Still expose runtime context (e.g., shell title, app title) in content pane.

---

## 4) Office Selector Label Simplification

UI text change in left-top office selector:

- Before: `🏢 Office: <name>`
- After: `🏢 <name>`

This reduces visual noise and matches the requested minimalist style.

---

## Verification

Commands run:

```bash
source ~/.cargo/env && cargo clippy --all-targets --all-features
source ~/.cargo/env && cargo test -q
```

Results:
- `cargo test -q`: all tests passing, 0 failures.
- `cargo clippy --all-targets --all-features`: warnings reduced to one structural warning (`module_inception`).

