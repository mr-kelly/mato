# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Idle Tab Detection**
  - Daemon tracks last PTY output time per tab (updated in read thread)
  - New `GetIdleStatus` / `IdleStatus` protocol messages
  - Client polls daemon every ~3 seconds (every 30 frames) for idle status
  - Tabs idle for ≥30 seconds show a `·` marker in the topbar
  - Tasks where all tabs are idle show a `·` marker in the sidebar
  - Threshold configurable via `IDLE_THRESHOLD_SECS` in `client/app.rs`

## [0.2.0] - 2026-02-21

### Added
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
