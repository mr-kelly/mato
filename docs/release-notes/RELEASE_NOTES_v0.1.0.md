# Release Notes - Mato v0.1.0

**Release Date**: 2026-02-20  
**Status**: Initial Release

## 🎉 First Release

Mato v0.1.0 is the initial release of Multi-Agent Terminal Office - a daemon-based persistent terminal multiplexer.

## ✨ Core Features

### Daemon Architecture
- **Persistent sessions** - Terminal sessions survive client restart
- **Unix socket communication** - Client-daemon protocol
- **Background daemon** - Runs continuously in background
- **PTY management** - Virtual terminal emulation

### Task & Tab Management
- **Task organization** - Group related terminals into tasks
- **Multiple tabs** - Each task can have multiple terminal tabs
- **State persistence** - Tasks and tabs saved to `~/.config/mato/state.json`
- **Auto-restore** - Previous session restored on startup

### User Interface
- **Beautiful TUI** - Built with ratatui, deep navy theme
- **Three focus modes** - Sidebar (tasks), Topbar (tabs), Content (terminal)
- **Mouse support** - Click to focus, scroll to navigate
- **Keyboard navigation** - Arrow keys, Enter, Esc

### Terminal Emulation
- **Pluggable emulators** - Support for vt100 and vte
- **Color support** - Full ANSI color rendering
- **Cursor positioning** - Accurate cursor display
- **Screen buffering** - Terminal content preserved

## 📋 Basic Operations

### Keyboard Shortcuts
- `n` - New task (in sidebar)
- `t` - New tab (in topbar)
- `w` - Close tab
- `x` - Close task
- `r` - Rename task/tab
- `q` - Quit
- `↑↓←→` - Navigate
- `Enter` - Focus terminal

### Daemon Commands
```bash
mato                    # Start client (auto-starts daemon)
mato --daemon          # Start daemon manually
mato --status          # Show daemon status
```

## 🏗️ Architecture

```
Client (TUI) ←→ Unix Socket ←→ Daemon (PTY Manager)
```

- **Client**: Terminal UI, user interaction
- **Daemon**: PTY processes, session management
- **Protocol**: JSON messages over Unix socket

## 📁 File Locations

- Config: `~/.config/mato/config.toml`
- State: `~/.config/mato/state.json`
- Socket: `~/.local/state/mato/daemon.sock`
- Logs: `~/.local/state/mato/daemon.log`

## 🎯 Known Limitations

- No scrollback buffer
- Terminal content lost on resize
- No session templates
- Basic error handling
- Limited keyboard shortcuts

## 📊 Statistics

- **Lines of Code**: ~2,500
- **Dependencies**: 15 crates
- **Test Coverage**: None
- **Documentation**: Basic README

## 🚀 What's Next

See [TODO.md](../todos/TODO.md) for planned features:
- Phase 4: Daemon improvements (lock file, signals, security)
- Phase 5: Code refactoring
- Phase 6: Testing infrastructure
- Phase 7: UI/UX improvements
- Phase 8: Onboarding system

---

**Installation**:
```bash
git clone https://github.com/mr-kelly/mato
cd mato
cargo build --release
```

**First Run**:
```bash
./target/release/mato
```
