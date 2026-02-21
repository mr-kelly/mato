# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- 🎯 **Activity Indicators** - Real-time spinners show which terminals are working
  - 10-frame Braille spinner animation (⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏)
  - Adaptive polling (12.5 FPS active, 5 FPS idle)
  - Smart logic: excludes current tab, only shows background activity
  - Perfect for monitoring AI agents and parallel tasks
- Context-aware Jump Mode help text (shows `q` shortcut when available)

### Fixed
- 🐛 **PTY Process Leak** - Closing tabs now properly kills PTY processes in daemon
  - Added `ClientMsg::ClosePty` protocol message
  - Daemon removes and drops PTY entries on close
  - Prevents memory leaks from accumulating bash processes
- Jump Mode help text now shows appropriate shortcuts based on focus

### Changed
- Replaced idle detection (30s threshold) with activity detection (2s threshold)
- Inverted UI logic: show "working" instead of "idle"
- README redesigned to highlight activity indicators

### Documentation
- Added `docs/ACTIVITY_INDICATORS.md` - Complete technical guide
- Added `docs/RECORDING_DEMO.md` - GIF recording instructions
- Updated README with "Why Mato?" and "Perfect For" sections
- Removed `docs/IDLE_DETECTION.md` (replaced by ACTIVITY_INDICATORS.md)

📖 **[Detailed Changelog →](docs/changelog/2026-02-22_activity-indicators.md)**

---

## [0.3.0] - 2026-02-21

**The Minimalist Release** 🎯

### Highlights
- 🚀 Jump Mode - EasyMotion-style navigation
- 🤖 AI-agent-friendly design (zero shortcut conflicts)
- ⌨️ Unified shortcuts (50% reduction)
- 🐛 Terminal content preserved on resize
- 🔧 Ctrl+Z suspend/resume fixed

📖 **[Full Release Notes →](docs/release-notes/RELEASE_NOTES_v0.3.0.md)**

## [0.2.0] - 2026-02-21

**Production Ready Release** ✅

### Highlights
- 🔒 Daemon improvements (lock file, signals, security)
- 🧪 Testing infrastructure (8 tests)
- 🎨 UI/UX improvements
- 🎯 6 workspace templates
- 📚 Comprehensive documentation

📖 **[Full Release Notes →](docs/release-notes/RELEASE_NOTES_v0.2.0.md)**  
📖 **[Technical Summary →](docs/release-notes/RELEASE_SUMMARY_v0.2.0.md)**

## [0.1.0] - 2026-02-20

**Initial Release** 🎉

### Highlights
- 🔄 Daemon-based persistence
- 📋 Task and tab management
- 🎨 Beautiful TUI with deep navy theme
- 🖱️ Mouse support
- ⌨️ Basic keyboard shortcuts

📖 **[Full Release Notes →](docs/release-notes/RELEASE_NOTES_v0.1.0.md)**

---

## Detailed Development History

For detailed development history, design decisions, and technical discussions, see:

- **[docs/changelog/](docs/changelog/)** - Dated development documents
- **[docs/release-notes/](docs/release-notes/)** - Release documentation
- **[AGENTS.md](AGENTS.md)** - AI-assisted development guide

## Version Links

[0.3.0]: https://github.com/YOUR_USERNAME/mato/releases/tag/v0.3.0
[0.2.0]: https://github.com/YOUR_USERNAME/mato/releases/tag/v0.2.0
[0.1.0]: https://github.com/YOUR_USERNAME/mato/releases/tag/v0.1.0
- **Phase 4: 100% TMUX Parity Achieved! 🎉**
  - Phase 4A: Critical daemon improvements
    - Lock file mechanism to prevent race conditions
    - Signal handling for graceful shutdown (SIGTERM, SIGINT)
    - Socket permissions set to 0700 for security
    - Graceful shutdown with resource cleanup
  - Phase 4B: Reliability improvements
    - PID file tracking daemon process
    - Event loop verification (Tokio after fork)
  - Phase 4C: Polish features
    - Enhanced status command with detailed information
    - Unified error handling system (`MatoError`)
    - Config reload on SIGHUP (hot reload)
    - Multiple clients support (shared sessions)

- **Phase 5: Code Refactoring**
  - Reorganized codebase into logical modules
  - Simplified main.rs from 338 to 184 lines (-45%)
  - Unified path management in `utils/paths.rs`
  - Clear separation: client/ daemon_modules/ utils/

- **Phase 6: Testing Infrastructure**
  - 10 unit tests (protocol, utils, config)
  - Test infrastructure with `src/lib.rs`
  - Terminal emulation test script

- **Phase 7: UI/UX Improvements**
  - Alt+1-9: Quick switch to tabs 1-9
  - Ctrl+PageUp/PageDown: Navigate tabs (works in any focus)
  - Complete keyboard shortcuts documentation

- **Phase 8: Onboarding System**
  - Beautiful TUI onboarding on first run
  - 6 workspace templates:
    - Power User (45 tasks, 250+ tabs) - User's actual config
    - Solo Developer (3 tasks, 8 tabs)
    - One-Person Company (4 tasks, 13 tabs)
    - Full-Stack Developer (4 tasks, 11 tabs)
    - Data Scientist (4 tasks, 11 tabs)
    - Minimal (1 task, 1 tab)
  - Templates embedded in binary (no external files needed)
  - Interactive template selection with descriptions

- Persist active task and active tab across restarts
- GitHub Actions for automated releases
- Installation script for quick setup
- Homebrew formula template
- Pre-built binaries for Linux and macOS (x86_64 and aarch64)
- --version flag support

### Changed
- Renamed `server.rs` to `daemon.rs` for consistency
- Improved error messages with context
- Enhanced logging with client IDs

### Technical
- 100% TMUX daemon/client capability parity
- Production-ready daemon architecture
- Comprehensive error handling
- Hot configuration reload
- Multi-client session sharing

## [0.1.0] - 2026-02-21

### Added
- Daemon-based persistence (sessions survive client restart)
- Multi-tab support with independent PTY sessions
- Task management (create, rename, close tasks)
- Tab management (create, rename, close tabs)
- Pluggable terminal emulator architecture
- vt100 emulator (default, basic ANSI)
- VTE emulator (better compatibility)
- Configuration file support (TOML)
- Status command (`mato --status`)
- Auto-start daemon
- State persistence (tasks and tabs)
- Mouse support (click, scroll, double-click)
- Keyboard-first navigation
- Beautiful TUI with deep navy theme
- Daemon and client logging

### Technical
- Two-layer pluggable architecture (providers + emulators)
- Unix socket communication (JSON protocol)
- Async daemon with tokio
- 76% TMUX daemon/client capability parity

### Documentation
- Comprehensive README
- TODO with roadmap
- TMUX daemon analysis
- Completion analysis vs TMUX
- Phase 4 implementation plan
- Refactoring plan

## [0.0.1] - 2026-02-20

### Added
- Initial prototype
- Basic terminal multiplexer
- Task and tab management
- Direct PTY provider

[Unreleased]: https://github.com/YOUR_USERNAME/mato/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/YOUR_USERNAME/mato/releases/tag/v0.1.0
[0.0.1]: https://github.com/YOUR_USERNAME/mato/releases/tag/v0.0.1
