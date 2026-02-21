# Mato

**Multi-Agent Terminal Office** — A daemon-based persistent terminal multiplexer with a beautiful TUI interface.

![Rust](https://img.shields.io/badge/rust-2021-orange?style=flat-square)
![ratatui](https://img.shields.io/badge/ratatui-0.29-blue?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)
![TMUX Parity](https://img.shields.io/badge/TMUX_parity-100%25-brightgreen?style=flat-square)
![Tests](https://img.shields.io/badge/tests-70_passing-brightgreen?style=flat-square)
![Coverage](https://img.shields.io/badge/coverage-65%25_logic-blue?style=flat-square)

## ✨ Features

- 🔄 **Daemon-based persistence** — Terminal sessions survive client restart (like tmux)
- 📋 **Task management** — Organize terminals into tasks with multiple tabs
- 🎨 **Beautiful TUI** — Modern interface with deep navy theme
- 🎯 **First-run onboarding** — Choose from 6 workspace templates
- 🔌 **Pluggable architecture** — Swappable terminal providers and emulators
- 🖱️ **Mouse support** — Click to focus, scroll to navigate, double-click to rename
- ⌨️ **Keyboard-first** — Full navigation without mouse (Alt+1-9, Ctrl+PageUp/Down)
- 💾 **State persistence** — Tasks and tabs automatically saved
- 🔥 **Hot reload** — Update config without restart (SIGHUP)
- 👥 **Multi-client** — Multiple clients can share the same session
- 🎯 **100% TMUX parity** — Production-ready daemon architecture

## 🚀 Quick Start

### Installation

```bash
# From source
git clone https://github.com/YOUR_USERNAME/mato
cd mato
cargo build --release
sudo mv target/release/mato /usr/local/bin/
```

### First Run

```bash
mato
```

On first run, you'll see an interactive onboarding screen to choose a workspace template:

- **Power User** (45 tasks, 250+ tabs) - Complete AI tools setup
- **Solo Developer** (3 tasks, 8 tabs) - Individual development
- **One-Person Company** (4 tasks, 13 tabs) - Business departments
- **Full-Stack Developer** (4 tasks, 11 tabs) - Multiple projects
- **Data Scientist** (4 tasks, 11 tabs) - Data analysis & ML
- **Minimal** (1 task, 1 tab) - Start from scratch

## 📖 Usage

### Basic Commands

```bash
# Start mato (auto-starts daemon if needed)
mato

# Check daemon status (shows uptime, clients, tabs)
mato --status

# Reload configuration without restart
kill -HUP $(cat ~/.local/state/mato/daemon.pid)

# Run daemon in foreground (for debugging)
mato --daemon --foreground
```

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New task |
| `Ctrl+T` | New tab |
| `Ctrl+W` | Close tab |
| `Ctrl+Q` | Quit |
| `Alt+1-9` | Quick switch to tab 1-9 |
| `Ctrl+PageUp/Down` | Previous/Next tab |
| `Tab` | Switch focus (sidebar ↔ terminal) |
| `↑↓` | Navigate tasks/tabs |
| `F2` | Rename task/tab |
| `Esc` | Cancel rename |

📖 **[Complete Keyboard Shortcuts Guide](docs/KEYBOARD_SHORTCUTS.md)**

### Configuration

Create `~/.config/mato/config.toml`:

```toml
# Choose terminal emulator
emulator = "vte"  # or "vt100" for basic ANSI
```

Hot reload after editing:
```bash
kill -HUP $(cat ~/.local/state/mato/daemon.pid)
```

### Emulator Selection

| Emulator | Compatibility | Best For |
|----------|---------------|----------|
| **vt100** (default) | Basic ANSI | General use, fast |
| **vte** | Better (GNOME Terminal / Alacritty) | Complex TUI apps (vim, htop) |

## ⌨️ Keybindings

### Global

| Key | Action |
|-----|--------|
| `Ctrl+Z` | Suspend (restore with `fg`) |

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

### Quick Shortcuts

| Shortcut | Action |
|----------|--------|
| `Alt+1-9` | Quick switch to tab 1-9 |
| `Ctrl+PageUp/PageDown` | Previous/Next tab |
| `n` / `t` | New task / New tab |
| `r` | Rename |
| `q` | Quit |

📖 **[Complete Keyboard Shortcuts Guide →](docs/KEYBOARD_SHORTCUTS.md)**

## 🏗️ Architecture

```
Client (UI) → DaemonProvider → Daemon → PtyProvider → TerminalEmulator
                    ↓ Unix Socket
            ~/.local/state/mato/daemon.sock
```

**Two-layer pluggable design:**
1. **Provider layer**: `PtyProvider` (direct) or `DaemonProvider` (persistent)
2. **Emulator layer**: `Vt100Emulator` or `VteEmulator` (configurable)

## 📁 File Locations

| File | Purpose |
|------|---------|
| `~/.config/mato/config.toml` | Configuration |
| `~/.config/mato/state.json` | Task and tab metadata |
| `~/.local/state/mato/daemon.sock` | Daemon communication |
| `~/.local/state/mato/daemon.log` | Daemon logs |
| `~/.local/state/mato/client.log` | Client logs |

## 🔧 Daemon Management

Mato uses a background daemon to persist terminal sessions:

- **Auto-start**: Daemon starts automatically when you run `mato`
- **Persistence**: Close the UI, sessions keep running
- **Reconnect**: Reopen `mato` to see your previous sessions
- **Status**: Run `mato --status` to check daemon state
- **Logs**: Check `~/.local/state/mato/daemon.log` for debugging

### Daemon Status

```bash
$ mato --status
✓ Daemon running
  Started: 2026-02-21 16:00:00
  Uptime: 2h 15m
  Active tabs: 8
  Socket: /home/user/.local/state/mato/daemon.sock
```

## 🎯 Roadmap

**Current Status**: 76% TMUX parity (see `docs/todos/completion-analysis.md`)

### Phase 4: Daemon Improvements (→ 100% TMUX Parity)
- [ ] Lock file mechanism (prevent race conditions)
- [ ] Signal handling (SIGTERM, SIGHUP)
- [ ] Socket permissions (security)
- [ ] Graceful shutdown
- [ ] PID file management
- [ ] Multiple clients support

### Phase 5: Code Refactoring
- [ ] Reorganize codebase structure
- [ ] Simplify main.rs
- [ ] Add utility modules

### Phase 6: UI/UX Improvements
- [ ] Tab reordering
- [ ] Split panes
- [ ] Copy mode
- [ ] Customizable keybindings

See [TODO.md](TODO.md) for complete roadmap.

## 🤝 Contributing

Contributions welcome! See [docs/todos/](docs/todos/) for implementation plans.

### Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Check daemon logs
tail -f ~/.local/state/mato/daemon.log
```

## 📚 Documentation

### User Guides
- [Keyboard Shortcuts](docs/KEYBOARD_SHORTCUTS.md) - Complete shortcuts reference
- [Idle Detection](docs/IDLE_DETECTION.md) - Idle tab and task markers
- [Templates Guide](templates/README.md) - Workspace templates

### Development
- [AGENTS.md](AGENTS.md) - AI-assisted development guide
- [CHANGELOG.md](CHANGELOG.md) - Version history
- [TODO](docs/todos/TODO.md) - Development roadmap
- [Development Changelog](docs/changelog/) - Historical development docs

## 📦 Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) — TUI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) — Terminal backend
- [portable-pty](https://github.com/wez/wezterm/tree/main/pty) — PTY management
- [vt100](https://github.com/doy/vt100-rust) — Terminal emulation
- [vte](https://github.com/jwilm/vte) — Advanced terminal parser
- [tokio](https://tokio.rs) — Async runtime
- [serde](https://serde.rs) — Serialization

## 📄 License

MIT

## 🙏 Acknowledgments

- Inspired by [tmux](https://github.com/tmux/tmux) daemon architecture
- Built with [ratatui](https://ratatui.rs) TUI framework
- Developed with AI assistance (see [AGENTS.md](AGENTS.md))
