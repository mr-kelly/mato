# matx

A keyboard-driven terminal multiplexer TUI — manage multiple tasks, each with independent terminal tabs, in a single beautiful interface.

![Rust](https://img.shields.io/badge/rust-2021-orange?style=flat-square)
![ratatui](https://img.shields.io/badge/ratatui-0.29-blue?style=flat-square)

## Features

- **Task sidebar** — create, close, rename, and navigate tasks
- **Multi-tab terminals** — each task has independent terminal tabs, each with its own PTY/bash session
- **Full terminal emulation** — vt100 rendering with ANSI colors, bold, italic, underline
- **Mouse support** — click to focus, scroll to navigate, double-click to rename
- **Rename anything** — tasks and tabs, inline popup editor
- **State persistence** — tasks and tabs saved to `~/.config/sandagent-tui/state.json` automatically
- **Deep navy theme** — RGB color scheme, modern look
- **Keyboard-first** — full navigation without mouse

## Install

```bash
git clone https://github.com/yourname/matx
cd matx
cargo build --release
./target/release/matx
```

## Keybindings

### Global

| Key | Action |
|-----|--------|
| `Ctrl+Z` | Suspend (restore on fg) |

### Sidebar (task list)

| Key | Action |
|-----|--------|
| `↑ / ↓` | Navigate tasks |
| `n` | New task |
| `x` | Close task |
| `r` | Rename task |
| `Enter` | Focus terminal |
| `q` | Quit |

### Topbar (tabs)

| Key | Action |
|-----|--------|
| `← / →` | Switch tab |
| `t` | New tab |
| `w` | Close tab |
| `r` | Rename tab |
| `Enter` | Focus terminal |
| `Esc` | Back to sidebar |

### Terminal (content)

| Key | Action |
|-----|--------|
| All keys | Forwarded to shell |
| `Esc` | Enter switch mode |
| `Esc` → `← / a` | Focus sidebar |
| `Esc` → `↑ / w` | Focus topbar |

### Mouse

| Action | Effect |
|--------|--------|
| Click task | Select task |
| Double-click task | Select + focus terminal |
| Click tab | Switch tab |
| Double-click tab | Rename tab |
| Click `＋` (topbar) | New tab |
| Click `＋` (sidebar) | New task |
| Scroll in sidebar | Navigate tasks |

### Rename popup

| Key | Action |
|-----|--------|
| `Enter` | Confirm |
| `Esc` | Cancel |

## State

Saved to `~/.config/sandagent-tui/state.json` on every task/tab change.

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) — TUI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) — terminal backend
- [portable-pty](https://github.com/wez/wezterm/tree/main/pty) — PTY/process management
- [vt100](https://github.com/doy/vt100-rust) — terminal emulation
- [serde](https://serde.rs) + [serde_json](https://github.com/serde-rs/json) — state persistence

## License

MIT
