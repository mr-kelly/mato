# 2026-02-22 — Themes, Settings UI, Update Check, Refactoring

## Theme System

Added a full theme system with three built-in themes and per-user overrides.

**Built-in themes:** `navy` (default), `gruvbox`, `catppuccin`

**Config file:** `~/.config/mato/theme.toml`

```toml
# Select a built-in theme
name = "gruvbox"

# Optionally override individual colors (RGB)
[colors]
accent = [255, 100, 100]
```

Only the fields you specify are overridden; the rest inherit from the base theme.

**Implementation:** `src/theme.rs` — `ThemeColors` struct, `load()`, `save_name()`, `builtin(name)`

All hardcoded color constants in `ui.rs` were replaced with `app.theme.*` calls.

---

## Settings TUI

A minimal in-app settings screen for switching themes.

- Press `s` in the sidebar to open
- `↑↓` to select theme, `Enter` to apply and save, `Esc` to close
- Theme change takes effect immediately (no restart needed)

---

## Update Check

The daemon periodically checks for a newer version of Mato.

**Flow:**
1. Daemon fetches `https://mato.sh/version.txt` on startup, then every hour
2. Compares remote version against `CARGO_PKG_VERSION`
3. Client queries daemon via `GetUpdateStatus` message on startup (after ~10s), then every hour
4. If a newer version is available, the statusbar shows a yellow notice on the right:
   `↑ Update available: x.y.z — mato.sh`
5. If the fetch fails, nothing is shown (debug log only)

**New protocol messages:** `ClientMsg::GetUpdateStatus`, `ServerMsg::UpdateStatus { latest: Option<String> }`

**New dependency:** `reqwest 0.12` (rustls-tls, no OpenSSL)

---

## Rename: `daemon_modules` → `daemon`

The `src/daemon_modules/` directory was renamed to `src/daemon/` for brevity. All import paths updated across `src/` and `tests/`.

---

## Rename: `Task` → `Desk`

The `Task` struct (and related: `SavedTask`, `tasks`, `active_task`, `close_task`, `begin_rename_task`) was renamed to `Desk` to better reflect the product concept. All source files and tests updated.

---

## i18n: Chinese → English

All Chinese text in source files and documentation was translated to English:
- `AGENTS.md` dialogue examples
- `src/daemon_modules/spawn.rs` comment (now `src/daemon/spawn.rs`)

---

## Product Name: `MATO` → `Mato`

All occurrences of the all-caps `MATO` in `.md`, `.rs`, and `.toml` files were normalized to `Mato`.

---

## Tests

- Updated all test helpers (`make_app`, `make_desk`) to use `App::new()` instead of manual struct construction, making them resilient to future field additions
- Fixed `resize_all_ptys_updates_dimensions` test to reflect deferred resize behavior (`pending_resize`)
- Updated `esc_mode`/`EscMode` references to `jump_mode`/`JumpMode`
- Renamed `q_in_topbar_does_not_quit` → `q_in_topbar_quits` to match intended behavior
- Restored missing Alt+1-9 tab switching logic in `input.rs`
- All 70 tests passing
