<div align="center">

# Mato

### 🏖️ Multi-Agent Terminal Office

**Monitor AI agents and tasks at a glance with real-time activity indicators**

[![Rust](https://img.shields.io/badge/rust-2021-orange?style=flat-square)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-8_passing-brightgreen?style=flat-square)](tests/)

[Why Mato?](#-why-mato) • [Perfect For](#-perfect-for) • [Features](#-features) • [Quick Start](#-quick-start)

</div>

---

<!-- Demo GIF - Shows activity indicators in action -->
<div align="center">
  <img src="docs/demo.gif" alt="Mato Activity Indicators Demo" width="800">
  <p><em>Real-time spinners show which terminals are working - perfect for AI agents and parallel tasks</em></p>
  <p><i>⚠️ GIF placeholder - Record demo showing spinner animation on active terminals</i></p>
</div>

---

## 🤔 Why Mato?

### The Problem: Lost in Terminal Chaos

When running multiple AI agents or long-running tasks:

| Challenge | Impact |
|-----------|--------|
| ❌ **Which terminal is still working?** | Constantly switching tabs to check |
| ❌ **Did my agent finish or hang?** | Wasting time on stuck processes |
| ❌ **Is that tab idle or just slow?** | No visual feedback on progress |
| ❌ **Keyboard conflicts with AI tools** | Claude Code, Cursor can't use Ctrl freely |

### Mato's Solution: Visual Progress Control

**See at a glance what's happening:**

```
┌─────────────────────────────────────────────────────────┐
│  Agent 1 ⠋    Agent 2    Agent 3 ⠴    Agent 4          │  ← Topbar
├─────────────────────────────────────────────────────────┤
│ ▶ Development ⠋  │                                      │
│   Testing        │  $ npm run dev                       │
│   Production ⠴   │  > Starting server...                │
│   Data Pipeline  │  ⠋ Compiling...                      │
└─────────────────────────────────────────────────────────┘
     ↑ Sidebar shows which tasks have active terminals
```

**Key Benefits:**

✅ **Real-time activity spinners** - Know exactly which terminals are busy  
✅ **Perfect for AI agents** - Monitor Claude, Codex, Copilot simultaneously  
✅ **Zero keyboard conflicts** - Only ONE special key (`Esc`)  
✅ **At-a-glance status** - No more tab-switching to check progress

---

## 🎯 Perfect For

### 🤖 AI Agent Workflows

```
┌─────────────────────────────────────────────────────────┐
│  Claude Agent ⠋    Codex CLI    GitHub Copilot ⠴       │
├─────────────────────────────────────────────────────────┤
│ ▶ AI Agents ⠋    │  $ claude "Build REST API"          │
│   Development    │  ⠋ Analyzing requirements...         │
│   Testing        │  ⠋ Generating code...                │
└─────────────────────────────────────────────────────────┘
```

**Use Cases:**
- Monitor multiple AI agents simultaneously
- Know when agents finish or hang
- Never miss completed tasks
- Zero interference with AI tool shortcuts

### 📊 Data Processing & ETL

```
┌─────────────────────────────────────────────────────────┐
│  ETL Pipeline ⠋    Database Sync    Report Gen ⠴       │
├─────────────────────────────────────────────────────────┤
│ ▶ Data Jobs ⠋    │  $ python etl_pipeline.py           │
│   Monitoring     │  Processing batch 3/10...            │
│   Backups        │  ⠋ 45% complete                      │
└─────────────────────────────────────────────────────────┘
```

**Use Cases:**
- Track long-running ETL jobs
- Monitor database migrations
- See progress across multiple pipelines
- Catch stuck processes immediately

### 🔧 Development & Testing

```
┌─────────────────────────────────────────────────────────┐
│  npm run dev ⠋    cargo watch    pytest ⠴              │
├─────────────────────────────────────────────────────────┤
│ ▶ Dev Servers ⠋  │  $ npm run dev                      │
│   Tests          │  ⠋ Webpack compiling...              │
│   Logs           │  Server running on :3000             │
└─────────────────────────────────────────────────────────┘
```

**Use Cases:**
- Monitor build processes and hot-reload
- Track test runs across multiple suites
- See which services are active
- Debug parallel development tasks

---

## ✨ Features
```

</td>
</tr>
</table>

## ✨ Features

<table>
<tr>
<td width="50%">

### 🎯 Activity Indicators
- **Real-time spinners** show working terminals
- Perfect for monitoring AI agents
- At-a-glance progress tracking
- Auto-adapts refresh rate (saves CPU)

### 🤖 AI-Agent Friendly
- **Zero shortcut conflicts**
- Claude Code, Cursor, Windsurf work perfectly
- All shell shortcuts preserved
- [Learn more →](docs/AI_AGENT_FRIENDLY.md)

### 🎨 Beautiful Interface
- Modern TUI with deep navy theme
- Mouse support (click, scroll, double-click)
- Visual feedback for all actions
- Smooth animations

</td>
<td width="50%">

### 🚀 Powerful Features
- **Jump Mode** - EasyMotion-style navigation
- **Daemon-based** - Sessions survive client restart
- **Multi-client** - Share sessions across terminals
- **6 Templates** - Power User, Solo Dev, Full-Stack, etc.

### 🔧 Developer Friendly
- Hot reload config (SIGHUP)
- Pluggable architecture
- Comprehensive tests
- Well-documented

### ⚡ Performance
- Adaptive polling (12.5 FPS active, 5 FPS idle)
- Minimal CPU usage when idle
- Efficient daemon architecture

</td>
</tr>
</table>

## 🆚 Comparison

| Feature | Traditional Multiplexers | Mato |
|---------|-------------------------|------|
| **Activity Indicators** | ❌ No visual feedback | ✅ Real-time spinners |
| **Progress Monitoring** | ❌ Manual checking | ✅ At-a-glance status |
| **Prefix Key** | Ctrl+B, Ctrl+A | ❌ None |
| **Special Keys** | 20+ shortcuts | ✅ Just `Esc` |
| **Shell Shortcuts** | ⚠️ Often conflicts | ✅ All preserved |
| **AI Agent Friendly** | ⚠️ Ctrl conflicts | ✅ Zero interference |
| **Learning Curve** | Steep | Gentle |
| **Navigation** | Prefix + arrows | Visual jump |

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

Choose from 6 workspace templates:
- **Power User** - 45 tasks, 250+ tabs (AI tools, dev, ops)
- **Solo Developer** - 3 tasks, 8 tabs (code, test, deploy)
- **Full-Stack Developer** - 4 tasks, 11 tabs (frontend, backend, db)
- **Data Scientist** - 4 tasks, 11 tabs (analysis, ML, viz)
- **One-Person Company** - 4 tasks, 13 tabs (dev, marketing, ops)
- **Minimal** - 1 task, 1 tab (start from scratch)

### Basic Usage

```bash
# In terminal, press Esc to navigate
Esc → a-z    # Jump to any task/tab
Esc → ←/↑    # Switch focus areas

# That's all you need to know!
```

## ⌨️ Keyboard Shortcuts

### Essential (You Only Need These)

| Key | Action | Description |
|-----|--------|-------------|
| `Esc` | Jump Mode | Navigate anywhere |
| `a-z` | Jump | In Jump Mode → instant jump |
| `n` | New | New task (sidebar) / tab (topbar) |
| `x` | Close | Close task (sidebar) / tab (topbar) |
| `r` | Rename | Rename task or tab |
| `q` | Quit | Exit Mato |

**That's it.** 6 keys for everything.

### Optional Convenience

| Key | Action |
|-----|--------|
| `↑↓←→` | Navigate lists |
| `Enter` | Focus terminal |

📖 **[Full keyboard reference →](docs/KEYBOARD_SHORTCUTS.md)**

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│  Client (TUI)                           │
│  - Beautiful interface                  │
│  - Jump Mode navigation                 │
│  - Mouse support                        │
└─────────────────┬───────────────────────┘
                  │ Unix Socket
┌─────────────────▼───────────────────────┐
│  Daemon (Background)                    │
│  - Persistent sessions                  │
│  - PTY management                       │
│  - Multi-client support                 │
└─────────────────────────────────────────┘
```

**Benefits**:
- Sessions survive client crashes
- Multiple clients can share one session
- Zero data loss on window resize
- Daemon runs continuously in background

## 📚 Documentation

<table>
<tr>
<td width="50%">

### 📖 User Guides
- **[Keyboard Shortcuts](docs/KEYBOARD_SHORTCUTS.md)** - Complete reference
- **[AI Agent Friendly](docs/AI_AGENT_FRIENDLY.md)** - Why Mato is perfect for AI assistants
- **[Terminal Persistence](docs/TERMINAL_PERSISTENCE.md)** - How sessions survive

</td>
<td width="50%">

### 🔧 Developer Docs
- **[AGENTS.md](AGENTS.md)** - AI-assisted development guide
- **[CHANGELOG.md](CHANGELOG.md)** - Version history
- **[Tests](tests/README.md)** - Testing guide

</td>
</tr>
</table>

## 🎨 Templates

Mato includes 6 pre-configured workspace templates:

<details>
<summary><b>Power User</b> - 45 tasks, 250+ tabs</summary>

Complete setup for AI-powered development:
- AI Tools (Claude, ChatGPT, Gemini, etc.)
- Development (Frontend, Backend, Mobile, etc.)
- DevOps (Docker, K8s, CI/CD, etc.)
- And much more...

</details>

<details>
<summary><b>Solo Developer</b> - 3 tasks, 8 tabs</summary>

Focused setup for individual developers:
- Development (Code, Test, Debug)
- Deployment (Build, Deploy, Monitor)
- Research (Docs, Search, Notes)

</details>

<details>
<summary><b>Full-Stack Developer</b> - 4 tasks, 11 tabs</summary>

Balanced setup for full-stack work:
- Frontend (React, Vue, etc.)
- Backend (API, Services, etc.)
- Database (SQL, Redis, etc.)
- DevOps (Deploy, Monitor, etc.)

</details>

📖 **[See all templates →](templates/README.md)**

## 🛠️ Advanced Usage

### Daemon Management

```bash
# Check daemon status
mato --status

# Run daemon in foreground (debugging)
mato --daemon --foreground

# Reload config without restart
kill -HUP $(cat ~/.local/state/mato/daemon.pid)
```

### Configuration

Terminal emulator config: `~/.config/mato/config.toml`

```toml
emulator = "vte"  # or "vt100"
```

Theme config: `~/.config/mato/theme.toml`

```toml
# Default (recommended): follow terminal/OS theme
name = "system"

# Optional: built-in themes
# name = "navy"
# name = "gruvbox"
# name = "catppuccin"
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run integration tests (requires daemon)
./tests/run_persistence_tests.sh
```

**Test coverage**: 8 tests (5 unit + 3 integration)
- Terminal persistence
- Content survival on resize
- Multi-client support

## 🗺️ Roadmap

**Current**: v0.2.0 - Production Ready ✅

**Future**:
- [ ] Scrollback buffer (Phase 9)
- [ ] Session management (Phase 10)
- [ ] Plugin system (Phase 11)
- [ ] Cloud sync (Phase 12)

📖 **[Full roadmap →](docs/todos/TODO.md)**

## 🤝 Contributing

Contributions welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- Built with [ratatui](https://ratatui.rs) TUI framework
- Terminal emulation via [vt100](https://crates.io/crates/vt100) and [vte](https://crates.io/crates/vte)
- Developed with AI assistance (see [AGENTS.md](AGENTS.md))

---

<div align="center">

**Made with 🏖️ for developers who value simplicity**

[⭐ Star us on GitHub](https://github.com/YOUR_USERNAME/mato) • [🐛 Report Bug](https://github.com/YOUR_USERNAME/mato/issues) • [💡 Request Feature](https://github.com/YOUR_USERNAME/mato/issues)

</div>

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

### The One Key You Need

**In terminal focus**: Press `Esc` to enter Jump Mode, then:
- `a-z` → Jump to any task/tab
- `←` or `↑` → Switch focus areas
- `Esc` → Cancel

**Everything else** goes directly to your shell. No prefix, no conflicts.

📖 **[Read why this matters for AI agents →](docs/AI_AGENT_FRIENDLY.md)**

### Optional Shortcuts

| Shortcut | Action | Context |
|----------|--------|---------|
| `Alt+1-9` | Jump to tab 1-9 | Anywhere |
| `Ctrl+PageUp/Down` | Previous/Next tab | Anywhere |
| `n` | New task/tab | Sidebar/Topbar |
| `x` | Close task/tab | Sidebar/Topbar |
| `r` | Rename | Sidebar/Topbar |
| `q` | Quit | Sidebar |

**Context-aware**: `n` and `x` adapt to where you are:
- Sidebar → Task operations
- Topbar → Tab operations
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

**Current Status**: v0.2.0 - Production ready! See [TODO.md](docs/todos/TODO.md) for future plans.

### Completed (v0.2.0)
- ✅ Lock file mechanism (prevent race conditions)
- ✅ Signal handling (SIGTERM, SIGHUP)
- ✅ Socket permissions (security)
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

- Built with [ratatui](https://ratatui.rs) TUI framework
- Terminal emulation via [vt100](https://crates.io/crates/vt100) and [vte](https://crates.io/crates/vte)
- Developed with AI assistance (see [AGENTS.md](AGENTS.md))
