# Mato v0.2.0 - 100% TMUX Parity! 🎉

We're excited to announce Mato v0.2.0, achieving **100% TMUX daemon/client parity**!

## 🎯 Highlights

### Production-Ready Daemon Architecture
- **Lock file mechanism** prevents race conditions
- **Signal handling** for graceful shutdown (SIGTERM, SIGINT, SIGHUP)
- **PID tracking** for process management
- **Multiple clients** can share the same session
- **Hot reload** configuration without restart

### Beautiful First-Run Experience
- **Interactive onboarding** with 6 workspace templates
- **Power User template** with 45 tasks and 250+ tabs
- Templates for developers, data scientists, and entrepreneurs
- All templates embedded in binary (no external files)

### Enhanced User Experience
- **Alt+1-9** for quick tab switching
- **Ctrl+PageUp/PageDown** for tab navigation
- **Enhanced status command** with detailed information
- **Unified error handling** with helpful messages

### Code Quality
- **45% reduction** in main.rs complexity (338 → 184 lines)
- **10 unit tests** covering core functionality
- **Modular architecture** with clear separation of concerns
- **Comprehensive documentation** including keyboard shortcuts guide

## 📦 Installation

### From Source
```bash
git clone https://github.com/mr-kelly/mato
cd mato
cargo build --release
sudo mv target/release/mato /usr/local/bin/
```

### Quick Start
```bash
mato  # First run shows template selection
```

## 🔄 Upgrading from v0.1.0

Your existing state will be preserved. Simply replace the binary:

```bash
# Backup your state (optional)
cp ~/.config/mato/state.json ~/.config/mato/state.json.backup

# Install new version
cargo install --git https://github.com/mr-kelly/mato --force

# Restart daemon
pkill -f "mato.*daemon"
mato
```

## 📝 Full Changelog

See [CHANGELOG.md](CHANGELOG.md) for complete details.

## 🙏 Acknowledgments

Thanks to all contributors and users who provided feedback!

## 🐛 Known Issues

None! This is a stable release.

## 📚 Documentation

- [README](README.md) - Getting started
- [Keyboard Shortcuts](docs/KEYBOARD_SHORTCUTS.md) - Complete reference
- [Templates Guide](templates/README.md) - Workspace templates
- [TODO](docs/todos/TODO.md) - Roadmap

---

**Full Changelog**: https://github.com/mr-kelly/mato/compare/v0.1.0...v0.2.0
