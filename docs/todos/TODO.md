# Mato Development TODO

## 🎉 Current Status: Production Ready!

**Version**: 0.3.0  
**Release Date**: 2026-02-21  
**Status**: Production Ready ✅

All Phase 4-8 objectives completed! See [Completed Phases](#completed-phases) below.

---

## 🚀 Immediate Next Steps

### 1. Idle Tab Detection ✅ (Completed 2026-02-21)
- Tabs/tasks show `·` marker when idle ≥30 seconds
- See `docs/changelog/2026-02-21_idle-tab-detection.md`

### 2. Release v0.2.0 (Priority: HIGH)
- [x] Commit all changes
- [x] Create git tag v0.2.0
- [x] Push to GitHub
- [x] Verify GitHub Actions build
- [x] Announce release

**Commands**:
```bash
git add -A
git commit -m "Release v0.2.0 - 100% TMUX Parity"
git tag -a v0.2.0 -m "Release v0.2.0 - 100% TMUX Parity"
git push origin main v0.2.0
```

### 2. Code Cleanup (Priority: MEDIUM)
- [x] Fix TODO in `vte_emulator.rs` - Implement CSI sequences
- [ ] Run `cargo clippy` and fix warnings
- [ ] Run `cargo fmt` for consistent formatting
- [ ] Review and remove dead code warnings

### 3. Post-Release (Priority: LOW)
- [ ] Collect user feedback
- [ ] Monitor GitHub issues
- [ ] Update documentation based on feedback
- [ ] Plan Phase 9 priorities

---

## 🚀 Future Development (Phase 9+)

### Phase 9: Advanced UI/UX 🎨
**Priority**: Medium  
**Estimated Effort**: 2-3 weeks

- [ ] **Tab Reordering** - Drag & drop tabs
- [ ] **Split Panes** - Horizontal/vertical splits
- [ ] **Copy Mode** - Scroll back buffer with vim-style navigation
- [ ] **Zoom Mode** - Fullscreen single tab
- [ ] **Tab Search/Filter** - Quick find tabs
- [ ] **Customizable Keybindings** - User-defined shortcuts
- [ ] **Vim-style Navigation** - Optional hjkl navigation
- [x] **Theme System** - Customizable colors

### Phase 10: Session Management 📦
**Priority**: Medium  
**Estimated Effort**: 2-3 weeks

- [ ] **Named Sessions** - Multiple independent sessions
- [ ] **Session Switching** - Switch between sessions
- [ ] **Session Templates** - Save/load session layouts
- [ ] **Import/Export** - Share session configurations
- [ ] **Session List** - View all active sessions
- [ ] **Attach/Detach** - Connect to existing sessions
- [ ] **Terminal State Persistence** - Save screen content to disk (see [FUTURE_STATE_PERSISTENCE.md](FUTURE_STATE_PERSISTENCE.md))
  - Serialize screen buffer, cursor position, colors
  - Restore on daemon restart (read-only view)
  - Offer restore scripts to re-run commands
  - **Note**: Cannot restore PTY process state (by design)

### Phase 11: Integration & Advanced Features 🔌
**Priority**: Low  
**Estimated Effort**: 3-4 weeks

- [ ] **SSH Integration** - Connect to remote hosts
- [ ] **Docker Support** - Attach to containers
- [ ] **Kubernetes** - Connect to pods
- [ ] **Cloud Shell** - AWS/GCP/Azure integration
- [ ] **Collaboration** - Share sessions with read-only mode
- [ ] **Recording** - Record and replay sessions
- [ ] **Plugins System** - Extensibility framework

### Phase 12: Performance & Polish 🔧
**Priority**: Low  
**Estimated Effort**: 1-2 weeks

- [ ] **Performance Profiling** - Benchmark and optimize
- [ ] **Memory Optimization** - Reduce memory footprint
- [ ] **Stress Testing** - Test with 1000+ tabs
- [ ] **Security Audit** - Review security practices
- [ ] **Accessibility** - Screen reader support
- [ ] **Internationalization** - Multi-language support

---

## 📚 Documentation Improvements (Priority: MEDIUM)

### Quick Wins
- [x] **Troubleshooting Guide** - Common issues and solutions (1 day)
- [ ] **Migration Guide** - From tmux/screen (1 day)
- [ ] **Performance Guide** - Optimization tips (1 day)

### Larger Projects
- [ ] **Video Tutorial** - Getting started guide (2-3 days)
- [ ] **Architecture Diagram** - Visual system overview (1 day)
- [ ] **API Documentation** - For plugin developers (2-3 days)

---

## 🧪 Testing Expansion (Priority: MEDIUM)

- [x] **Integration Tests** - End-to-end scenarios
- [ ] **Performance Tests** - Benchmarking suite
- [ ] **Stress Tests** - Test with 1000+ tabs
- [ ] **Fuzzing** - Security testing
- [ ] **CI/CD Enhancement** - Automated testing pipeline

---

## 📦 Distribution (Priority: LOW)

- [ ] **Homebrew** - Official tap
- [ ] **AUR** - Arch Linux package
- [ ] **Debian/Ubuntu** - .deb packages
- [ ] **Fedora/RHEL** - .rpm packages
- [ ] **Snap** - Universal Linux package
- [ ] **Docker Image** - Containerized version

---

## 🎯 Recommended Priority Order

### Immediate (This Week)
1. ✅ **Release v0.2.0** - Push to production
2. 🧹 **Code Cleanup** - Fix warnings, format code
3. 📝 **Collect Feedback** - Monitor issues and discussions

### Short-term (Next 1-2 Months)
4. 📚 **Documentation** - Troubleshooting, migration guides
5. 🧪 **Testing** - Integration tests, stress tests
6. 🎨 **Phase 9 Quick Wins** - Zoom mode, tab search

### Medium-term (Next 3-6 Months)
7. 🎨 **Phase 9 Complete** - Split panes, copy mode, themes
8. 📦 **Phase 10** - Session management
9. 📦 **Distribution** - Homebrew, AUR packages

### Long-term (Next 6-12 Months)
10. 🔌 **Phase 11** - SSH, Docker, K8s integration
11. 🔧 **Phase 12** - Performance optimization, security audit
12. 🚀 **v1.0.0** - Production release with all features

---

## ✅ Completed Phases

<details>
<summary><b>Phase 1-8: Foundation to 100% TMUX Parity (Click to expand)</b></summary>

---

## Phase 6: UI/UX Improvements 🎨

### Terminal Emulation
- [ ] Test with complex TUI apps (vim, htop, neovim)
- [ ] Fix any rendering issues
- [ ] Add more emulator options if needed

### UI Features
- [ ] Tab reordering (drag & drop)
- [ ] Tab search/filter
- [ ] Split panes (horizontal/vertical)
- [ ] Zoom mode (fullscreen tab)
- [ ] Copy mode (scroll back buffer)

### Keyboard Shortcuts
- [ ] Customizable keybindings
- [ ] Vim-style navigation option
- [x] Quick tab switching (Alt+1-9)

---

## Phase 7: Advanced Features 🚀

### Session Management
- [ ] Named sessions (like tmux sessions)
- [ ] Session switching
- [ ] Session templates
- [ ] Import/export sessions

### Collaboration
- [ ] Share session URL
- [ ] Read-only mode for viewers
- [ ] Multi-cursor editing

### Integration
- [ ] SSH integration
- [ ] Docker container support
- [ ] Kubernetes pod attach
- [ ] Cloud shell integration

---

## Documentation 📚

### User Documentation
- [x] Installation guide
- [x] Quick start tutorial
- [x] Configuration reference
- [x] Keyboard shortcuts cheatsheet
- [x] Troubleshooting guide

### Developer Documentation
- [x] Architecture overview
- [ ] Contributing guide
- [ ] API documentation
- [x] Testing guide

---

## Testing & Quality 🧪

### Unit Tests
- [x] Protocol serialization/deserialization
- [ ] Emulator implementations
- [x] Provider implementations
- [x] Utility functions

### Integration Tests
- [x] Client-daemon communication
- [x] Session persistence
- [x] Multi-tab scenarios
- [x] Error handling

### Performance Tests
- [ ] Benchmark rendering speed
- [ ] Memory usage profiling
- [ ] CPU usage profiling
- [ ] Stress test (100+ tabs)

---

## Release Preparation 📦

### Version 0.1.0 (MVP)
- [x] Basic daemon/client architecture
- [x] Multi-tab support
- [x] Session persistence
- [x] Multiple emulators
- [x] Phase 4A (critical fixes)
- [x] Phase 5 (refactoring)
- [x] Basic documentation

### Version 0.2.0 (Stable)
- [x] Phase 4B (reliability)
- [x] Phase 4C (polish)

### Phase 1: Daemon Provider ✅
- Full daemon-based persistence
- Client auto-starts daemon
- Multi-tab support with independent PTY sessions
- Performance optimized (10fps refresh rate)

### Phase 2: Multiple Terminal Emulators ✅
- `TerminalEmulator` trait for pluggable backends
- `Vt100Emulator` - basic ANSI support
- `VteEmulator` - better compatibility (GNOME Terminal/Alacritty parser)
- Configuration file support (TOML)

### Phase 3: Advanced Features ✅
- `mato --status` command
- ⚡ indicator in topbar shows daemon connection
- Stale socket cleanup
- Active tabs count logging

### Phase 4: Daemon Improvements ✅
**See**: `phase4-implementation-plan.md` for detailed plan

- Phase 4A: Lock file, signal handling, socket permissions, graceful shutdown
- Phase 4B: PID file, event loop verification
- Phase 4C: Enhanced status, error handling, config reload, multi-client

### Phase 5: Code Refactoring ✅
**See**: `refactoring-plan.md` for detailed plan

- Extracted utilities (paths, logging)
- Moved client code to `client/`
- Moved daemon code to `daemon_modules/`
- Simplified `main.rs` from 338 to 184 lines (-45%)
- Unified path management

### Phase 6: Testing ✅
- 10 unit tests (protocol, utils, config)
- Test infrastructure (`src/lib.rs`)
- Terminal emulation test script

### Phase 7: UI/UX Improvements ✅
- Alt+1-9: Quick switch to tabs
- Ctrl+PageUp/PageDown: Navigate tabs
- Complete keyboard shortcuts documentation

### Phase 8: Onboarding System ✅
- First-run detection
- 6 workspace templates (Power User, Solo Developer, etc.)
- Beautiful TUI template selector
- Templates embedded in binary

</details>

---

## 📝 Notes

### Current Persistence Behavior

**What persists across client restart**: ✅
- PTY process (bash continues in daemon)
- Terminal content (in daemon memory)
- Running commands continue

**What does NOT persist across daemon restart**: ❌
- PTY process (killed with daemon)
- Terminal content (lost with daemon)
- All process state

**See**: [TERMINAL_PERSISTENCE.md](../TERMINAL_PERSISTENCE.md) for details  
**Future**: [FUTURE_STATE_PERSISTENCE.md](FUTURE_STATE_PERSISTENCE.md) for Phase 10 design

### File Locations
- Daemon log: `~/.local/state/mato/daemon.log`
- Client log: `~/.local/state/mato/client.log`
- Socket: `~/.local/state/mato/daemon.sock`
- PID file: `~/.local/state/mato/daemon.pid`
- Lock file: `~/.local/state/mato/daemon.lock`
- Config: `~/.config/mato/config.toml`
- State: `~/.config/mato/state.json`

### Configuration
```toml
# ~/.config/mato/config.toml
emulator = "vte"  # or "vt100"
```

### Hot Reload
```bash
kill -HUP $(cat ~/.local/state/mato/daemon.pid)
```

### References
- [TMUX Analysis](tmux-daemon-analysis.md) - Source code insights
- [Completion Analysis](completion-analysis.md) - Feature comparison
- [Phase 4 Plan](phase4-implementation-plan.md) - Daemon improvements
- [Refactoring Plan](refactoring-plan.md) - Code reorganization
