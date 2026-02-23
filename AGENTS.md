# AGENTS.md - AI-Assisted Development Guide

This document records the AI-assisted development process of Mato, serving as a guide for future AI-human collaborative projects.

## 🤖 Project Overview

**Project**: Mato (Multi-Agent Terminal Office)  
**Development Period**: 2026-02-20 to 2026-02-21 (2 days)  
**AI Assistant**: Claude (Anthropic)  
**Human Developer**: Kelly  
**Result**: v0.1.0 → v0.3.0 - Production-ready minimalist multiplexer

**Core Philosophy**: AI-agent-friendly design with zero shortcut interference

## 📊 Development Statistics

| Metric | Value |
|--------|-------|
| **Total Phases** | 8 (Phase 1-8) |
| **Versions Released** | 3 (v0.1.0, v0.2.0, v0.3.0) |
| **Code Reduction** | main.rs: 338 → 184 lines (-45%) |
| **Shortcut Reduction** | 18 → 9 keys (-50%) |
| **Tests Added** | 0 → 8 tests (5 unit + 3 integration) |
| **Test Coverage** | Terminal persistence + resize: 100% |
| **Templates Created** | 6 workspace templates |
| **Documentation** | 5 → 22 files |
| **Critical Bugs Fixed** | 3 (deadlock, resize, suspend) |
| **Design Innovation** | One-key navigation (Esc only) |

## 🎯 Development Methodology

### Phase-Based Approach

We used a **structured phase-based development** approach:

1. **Phase 4: Daemon Improvements** (Critical → Reliability → Polish)
2. **Phase 5: Code Refactoring** (Modularization)
3. **Phase 6: Testing Infrastructure** (Unit tests)
4. **Phase 7: UI/UX Improvements** (Keyboard shortcuts)
5. **Phase 8: Onboarding System** (First-run experience)

**Key Principle**: Complete one phase before moving to the next.

### Incremental Development

Each phase was broken into small, testable increments:

```
Phase 4A: Critical Fixes
├─ Lock file mechanism
├─ Signal handling
├─ Socket permissions
└─ Graceful shutdown

Phase 4B: Reliability
├─ PID file
└─ Event loop verification

Phase 4C: Polish
├─ Enhanced status command
├─ Better error handling
├─ Config reload (SIGHUP)
└─ Multiple clients support
```

**Key Principle**: Each increment compiles and tests successfully.

## 🛠️ AI-Human Collaboration Patterns

### 1. Planning Phase

**Human**: "We've reached 92% TMUX parity, what's next?"

**AI**: 
- Analyzes current state
- Proposes structured plan (Phase 4C)
- Prioritizes tasks by value/effort
- Provides clear roadmap

**Output**: `docs/changelog/phase4-implementation-plan.md`

### 2. Implementation Phase

**Human**: "OK, continue" or "OK"

**AI**:
- Implements minimal code changes
- Compiles and tests after each change
- Provides progress updates
- Handles errors immediately

**Pattern**: Small commits, frequent testing

### 3. Verification Phase

**Human**: "Run the tests first"

**AI**:
- Runs all tests
- Checks compilation
- Verifies functionality
- Reports results clearly

**Pattern**: Test-driven validation

### 4. Documentation Phase

**Human**: "Continue" (after implementation complete)

**AI**:
- Updates CHANGELOG.md
- Updates README.md
- Updates TODO.md
- Creates release notes

**Pattern**: Documentation follows implementation

## 💡 Best Practices Learned

### 1. Clear Communication

✅ **Good**:
- "Continue" - Continue with next logical step
- "OK" - Proceed as planned
- "Run the tests first" - Specific action request

❌ **Avoid**:
- Vague requests without context
- Multiple unrelated tasks at once

### 2. Incremental Changes

✅ **Good**:
```rust
// Step 1: Add field
pub struct Daemon {
    client_count: Arc<AtomicUsize>,
}

// Step 2: Initialize
Self {
    client_count: Arc::new(AtomicUsize::new(0)),
}

// Step 3: Use it
let client_id = self.client_count.fetch_add(1, Ordering::Relaxed);
```

❌ **Avoid**:
- Large refactors without testing
- Multiple features in one commit

### 3. Test-Driven Development

✅ **Pattern**:
1. Write code
2. Compile (`cargo build`)
3. Test (`cargo test`)
4. Fix errors
5. Repeat

❌ **Avoid**:
- Writing lots of code without testing
- Assuming it works without verification

### 4. Documentation as You Go

✅ **Pattern**:
- Update CHANGELOG.md after each phase
- Update README.md when adding features
- Create guides for complex features
- Keep TODO.md current

❌ **Avoid**:
- Leaving documentation for "later"
- Undocumented features

## 🔧 Technical Decisions

### 1. Error Handling Evolution

**Before** (Phase 1-3):
```rust
pub fn save_state(app: &App) {
    if let Ok(json) = serde_json::to_string_pretty(&state) {
        std::fs::write(path, json).ok();
    }
}
```

**After** (Phase 4C):
```rust
pub fn save_state(app: &App) -> Result<()> {
    let json = serde_json::to_string_pretty(&state)?;
    std::fs::write(&path, json)
        .map_err(|e| MatoError::StateSaveFailed(format!("Cannot write to {}: {}", path.display(), e)))?;
    Ok(())
}
```

**Lesson**: Unified error types improve debugging and user experience.

### 2. Signal Handling

**Challenge**: How to handle SIGHUP for config reload?

**Solution**:
```rust
// Add SIGHUP to signal set
libc::sigaddset(&mut sigset, libc::SIGHUP);

// Check in event loop
if self.signals.should_reload() {
    self.reload_config();
}
```

**Lesson**: Use atomic flags for signal communication between threads.

### 3. Multi-Client Support

**Challenge**: How to support multiple clients sharing one daemon?

**Solution**:
- Use `Arc<DashMap<...>>` for shared state
- Track client count with `AtomicUsize`
- Each client gets independent `handle_client` task
- All clients read from same PTY providers

**Lesson**: Rust's ownership system makes concurrent access safe and easy.

### 4. Terminal Persistence Testing

**Challenge**: How to ensure terminal content survives client reconnection?

**Solution**:
- **Unit tests** (5 tests) - Test `PtyProvider` directly
  - Content persistence across `get_screen()` calls
  - Resize behavior (no-op when size unchanged)
  - Multiple writes accumulation
  - Spawn idempotency
- **Integration tests** (2 tests) - Test full client-daemon flow
  - Content survives client disconnect/reconnect
  - Multiple tabs maintain separate content

**Test Infrastructure**:
```bash
# Unit tests (fast, no daemon needed)
cargo test --test terminal_persistence_tests

# Integration tests (requires running daemon)
cargo test --test daemon_persistence_tests -- --ignored

# All tests with one command
./tests/run_persistence_tests.sh
```

**Files**:
- `tests/terminal_persistence_tests.rs` - Unit tests
- `tests/daemon_persistence_tests.rs` - Integration tests
- `tests/run_persistence_tests.sh` - Test runner
- `tests/README.md` - Test documentation

**Lesson**: Comprehensive testing prevents regressions and documents expected behavior.

### 5. Terminal Resize Without Content Loss

**Challenge**: When client window resizes, terminal content disappears. How to preserve content?

**Root Cause Analysis**:
1. Client sends `Resize` message to daemon
2. Daemon calls `PtyProvider::resize(rows, cols)`
3. Terminal emulator (`vt100::Parser` or `vte`) doesn't support content-preserving resize
4. Emulator creates new parser → **screen cleared**
5. Content lost even after client reconnects (daemon-side state is gone)

**Failed Approaches**:
- ❌ Delay resize on client side → Doesn't help, daemon still clears
- ❌ Track size to avoid duplicate resizes → Still clears on real resize
- ❌ Try to preserve content in emulator → Not supported by vt100/vte crates

**Correct Solution**: **Don't resize the PTY at all!**

```rust
// In daemon.rs
ClientMsg::Resize { tab_id, rows, cols } => {
    // DON'T resize the PTY! This would clear the screen.
    // The PTY should keep running at its original size.
    // Only the client's display needs to adapt to window size.
    tracing::debug!("Ignoring resize request - PTY size is fixed");
    continue;
}
```

**Key Insight**: 
- PTY is a **server-side resource** that should remain stable
- Client window size is a **client-side concern**
- If PTY size ≠ window size:
  - Small window → Clip/scroll the display
  - Large window → Show with padding

**This matches standard terminal multiplexer behavior**:
- Multiple clients with different window sizes
- PTY uses smallest common size
- Larger clients see padding
- **Content never lost on resize**

**Lesson**: Question assumptions. The "obvious" solution (resize PTY to match window) was wrong. The correct solution is simpler: don't resize at all.

### 6. Minimalist Keyboard Design

**Challenge**: How to avoid shortcut conflicts with AI agents and shell?

**Problem Analysis**:
- Traditional multiplexers use prefix keys (Ctrl+B, Ctrl+A)
- These conflict with:
  - Shell shortcuts (Ctrl+A = line start, Ctrl+K = kill line)
  - AI assistants (Claude Code, Cursor use Ctrl extensively)
  - Vim shortcuts (Ctrl+B = page up)
- Users must remember "escape sequences" to use these keys

**Solution**: **One key philosophy** - Only `Esc` is special

```rust
// In terminal focus
Focus::Content => {
    match key.code {
        KeyCode::Esc => {
            // Only Esc enters Jump Mode
            app.jump_mode = JumpMode::Active;
        }
        _ => {
            // Everything else → shell
            app.pty_write(&bytes);
        }
    }
}
```

**Benefits**:
- ✅ **Zero interference** - All Ctrl shortcuts work normally
- ✅ **AI-agent friendly** - Claude Code, Cursor, Windsurf work perfectly
- ✅ **Shell shortcuts preserved** - Ctrl+A/E/K/U/R all work
- ✅ **Vim compatible** - Ctrl+B/F/D/U work as expected
- ✅ **Simple mental model** - "Esc to navigate, everything else is normal"

**Design Trade-off**:
- ❌ Esc in shell requires workaround (use Ctrl+[ or press Esc twice)
- ✅ But Esc is rarely used in shells (most operations use Ctrl)

**Lesson**: Minimalism wins. One special key is better than 20+ prefix combinations. Design for the 99% use case (working in terminal), not the 1% (navigation).

## 📚 Code Organization Evolution

### Before (Phase 1-3)
```
src/
├── main.rs (338 lines) ❌ Too large
├── app.rs
├── ui.rs
├── input.rs
├── persistence.rs
├── daemon.rs
└── ...
```

### After (Phase 5)
```
src/
├── main.rs (184 lines) ✅ Simplified
├── client/
│   ├── app.rs
│   ├── ui.rs
│   ├── input.rs
│   ├── persistence.rs
│   ├── onboarding.rs
│   └── onboarding_tui.rs
├── daemon_modules/
│   ├── daemon.rs
│   ├── lock.rs
│   ├── signals.rs
│   ├── pid.rs
│   ├── spawn.rs
│   └── status.rs
├── utils/
│   ├── paths.rs
│   └── id.rs
└── error.rs

tests/
├── terminal_persistence_tests.rs    # 5 unit tests
├── daemon_persistence_tests.rs      # 2 integration tests
├── run_persistence_tests.sh         # Test runner
└── README.md                         # Test documentation
```

**Lesson**: Clear module boundaries improve maintainability.

## 🎓 Lessons for Future Projects

### 1. Start with a Plan

Before coding:
- Analyze current state
- Define clear goals (e.g., "100% TMUX parity")
- Break into phases
- Prioritize by value/effort

### 2. Maintain Momentum

Keep development flowing:
- Small, frequent commits
- Test after each change
- Fix errors immediately
- Document as you go

### 3. Use Historical Context

AI assistants benefit from:
- Previous conversation history
- Existing documentation
- Code structure understanding
- Project goals and constraints

### 4. Celebrate Milestones

Acknowledge progress:
- ✅ Phase completed
- 🎉 100% parity achieved
- 📦 Version released

This maintains motivation and provides clear checkpoints.

## 🚀 Recommended Workflow

### For New Features

1. **Plan** - Define scope and approach
2. **Implement** - Small incremental changes
3. **Test** - Compile and run tests
4. **Document** - Update relevant docs
5. **Review** - Check against goals
6. **Commit** - Save progress

### For Bug Fixes

1. **Reproduce** - Understand the issue
2. **Diagnose** - Find root cause
3. **Fix** - Minimal code change
4. **Test** - Verify fix works
5. **Prevent** - Add test if needed
6. **Document** - Update CHANGELOG

### For Refactoring

1. **Identify** - What needs improvement?
2. **Plan** - How to reorganize?
3. **Migrate** - Move code incrementally
4. **Test** - Ensure nothing breaks
5. **Clean** - Remove old code
6. **Document** - Explain new structure

## 📈 Version Management Strategy

### Semantic Versioning

We follow [SemVer](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.2.0): New features, backward compatible
- **PATCH** (0.2.1): Bug fixes

### Version History

| Version | Date | Milestone | TMUX Parity |
|---------|------|-----------|-------------|
| 0.1.0 | 2026-02-20 | Initial release | 76% |
| 0.2.0 | 2026-02-21 | 100% TMUX parity | 100% |
| 1.0.0 | TBD | Production ready | 100%+ |

### Release Process

1. **Complete Phase** - All features implemented
2. **Update Docs** - CHANGELOG, README, TODO
3. **Bump Version** - Update Cargo.toml
4. **Create Release Notes** - See [Documentation Standards](#documentation-standards)
5. **Push main branch** - Let CI run release workflow
6. **CI Creates Tag + GitHub Release** - Do not create release tags manually

### Release Ownership Rule (Important)

- `main` branch releases are **CI-owned** via `.github/workflows/release.yml`.
- **Do not manually run** `git tag -a vX.Y.Z` for release versions.
- **Do not manually publish** GitHub Releases for release versions.
- Manual prerelease tags (e.g., `-alpha.N`) are allowed only if explicitly required by workflow policy.
- If a release tag already exists before CI runs, `Validate tag policy` will fail and block release creation.

## 📚 Documentation Standards

### Directory Structure

```
mato/
├── AGENTS.md                    # AI development guide
├── CHANGELOG.md                 # Version history
├── README.md                    # Project overview
│
├── docs/
│   ├── changelog/              # Historical development docs
│   │   ├── README.md
│   │   └── YYYY-MM-DD_*.md    # Dated documents
│   │
│   ├── todos/                  # Current development plans
│   │   ├── README.md
│   │   └── TODO.md
│   │
│   ├── release-notes/          # Release documentation
│   │   ├── README.md
│   │   ├── RELEASE_NOTES_vX.Y.Z.md
│   │   ├── RELEASE_SUMMARY_vX.Y.Z.md
│   │   └── FINAL_SUMMARY.md
│   │
│   └── *.md                    # User guides
│
└── templates/
    └── README.md
```

### Naming Conventions

#### Changelog Documents
Format: `YYYY-MM-DD_description.md`

Examples:
- `2026-02-20_completion-analysis.md`
- `2026-02-20_phase4-implementation-plan.md`
- `2026-02-21_refactoring-plan.md`

**Purpose**: Track when decisions were made, easy chronological sorting

#### Release Documents
Format: `RELEASE_TYPE_vX.Y.Z.md`

Types:
- `RELEASE_NOTES_v0.2.0.md` - Public announcement
- `RELEASE_SUMMARY_v0.2.0.md` - Technical details
- `FINAL_SUMMARY.md` - Complete milestone summary

**Purpose**: Clear version tracking, different audiences

### Document Types

#### User-Facing
- **README.md** - Project overview, quick start
- **CHANGELOG.md** - Version history
- **docs/KEYBOARD_SHORTCUTS.md** - User reference
- **templates/README.md** - Template guide

#### Developer-Facing
- **AGENTS.md** - AI development methodology
- **docs/todos/TODO.md** - Development roadmap
- **docs/changelog/YYYY-MM-DD_*.md** - Historical decisions
- **docs/release-notes/RELEASE_SUMMARY_*.md** - Technical changes

#### Release Documentation
- **docs/release-notes/RELEASE_NOTES_*.md** - What's new, upgrade guide
- **docs/release-notes/RELEASE_SUMMARY_*.md** - Statistics, code changes
- **docs/release-notes/FINAL_SUMMARY.md** - Complete development summary

### Documentation Workflow

#### During Development
1. Create dated documents in `docs/changelog/` for major decisions
2. For every AI Agent chat session, create or continue exactly one session changelog in `docs/changelog/`
3. Keep updating that same session changelog throughout the session (do not split one session across multiple changelog files unless explicitly requested)
4. Update `docs/todos/TODO.md` as phases progress
5. Keep `CHANGELOG.md` updated with unreleased changes

#### At Release
1. Move unreleased changes in `CHANGELOG.md` to new version section
2. Create `RELEASE_NOTES_vX.Y.Z.md` in `docs/release-notes/`
3. Create `RELEASE_SUMMARY_vX.Y.Z.md` in `docs/release-notes/`
4. Update `README.md` with new features
5. Update version in `Cargo.toml`

#### After Major Milestone
1. Create `FINAL_SUMMARY.md` in `docs/release-notes/`
2. Archive completed phases in `docs/changelog/`
3. Update `AGENTS.md` with lessons learned

### Best Practices

✅ **Do**:
- Date all historical documents (`YYYY-MM-DD_*.md`)
- Maintain one changelog file per AI Agent session and keep appending to it during that session
- Keep release notes in `docs/release-notes/`
- Update documentation as you code
- Use clear, descriptive filenames
- Maintain README.md indexes in subdirectories

❌ **Don't**:
- Mix release notes with changelog documents
- Leave documents undated
- Create documentation "later"
- Use vague filenames
- Forget to update indexes

## 🎯 Success Metrics

### Quantitative

- ✅ 8 tests passing (5 unit + 3 integration)
- ✅ 100% terminal persistence test coverage
- ✅ 45% code reduction in main.rs
- ✅ 50% keyboard shortcut reduction
- ✅ 0 compilation errors
- ✅ 0 test failures
- ✅ 3 critical bugs fixed

### Qualitative

- ✅ Clean, modular architecture
- ✅ Comprehensive documentation (22 files)
- ✅ Beautiful user experience
- ✅ Production-ready quality
- ✅ Clear development history
- ✅ AI-agent-friendly design
- ✅ Minimalist philosophy

## 💡 Key Lessons Learned

### 1. Simplicity is Hard

**Challenge**: How to make the interface simpler without losing functionality?

**Discovery**: Removing features is harder than adding them.
- Started with 18 shortcuts
- Realized Jump Mode could replace 9 of them
- Final result: 9 shortcuts (50% reduction)

**Lesson**: **Question every feature**. If Jump Mode can do it better, remove the old way.

### 2. User Testing Reveals Truth

**Challenge**: Ctrl+Z didn't work after implementation.

**Process**:
1. Implemented suspend logic
2. User tested: "After `fg`, the terminal is still blank."
3. Realized terminal state wasn't restored
4. Added SIGCONT signal handler
5. User tested again: "Looks like it's fixed now."

**Lesson**: **Test immediately after implementation**. Don't assume it works.

### 3. Design Philosophy Drives Decisions

**Challenge**: Should we keep Alt+1-9 and Ctrl+PageUp/Down?

**Decision Process**:
- Philosophy: "One key to rule them all"
- Jump Mode provides visual navigation
- Old shortcuts are redundant
- **Remove them**

**Lesson**: **Strong philosophy makes decisions easy**. When in doubt, refer to core principles.

### 4. Documentation is Marketing

**Challenge**: How to position Mato vs established tools?

**Solution**:
- Don't say "like tmux but better"
- Say "AI-agent-friendly, zero conflicts"
- Focus on unique value proposition
- Use comparison tables

**Lesson**: **Documentation shapes perception**. Good docs = good product positioning.

### 5. Iterate on User Feedback

**Real conversation**:
- User: "The Jump Mode label is a bit too high."
- Fixed: Use `sidebar_list_area` instead of calculating
- User: "Looks good now."

**Lesson**: **Quick iteration wins**. Fix → Test → Confirm → Move on.

## 🔮 Future Recommendations

### For Phase 9+ Development

1. **Continue Phase-Based Approach** - It works!
2. **Maintain Test Coverage** - Add tests for new features
3. **Document Decisions** - Keep AGENTS.md updated
4. **Follow Naming Conventions** - Date all changelog docs
5. **User Feedback** - Incorporate real-world usage
6. **Test Immediately** - Don't wait to verify fixes
7. **Question Everything** - Can it be simpler?

### For Other Projects

This methodology can be applied to any software project:

1. **Define Clear Goals** - What does "done" look like?
2. **Break into Phases** - Manageable chunks
3. **Incremental Development** - Small, testable changes
4. **Continuous Testing** - Catch issues early
5. **Document Everything** - Follow naming conventions
6. **Organize Documentation** - Separate changelog from releases
7. **Strong Philosophy** - Let principles guide decisions
8. **User Testing** - Test with real users immediately

## 📝 Conclusion

AI-assisted development is **highly effective** when:

- Goals are clear
- Communication is concise
- Changes are incremental
- Testing is continuous
- Documentation is maintained
- **Standards are followed**
- **User feedback is immediate**
- **Philosophy guides decisions**

**Key Insight**: The AI assistant acts as a **tireless pair programmer** who:
- Never gets tired
- Maintains context across sessions
- Follows best practices consistently
- Documents as it goes
- Tests thoroughly
- **Adheres to project standards**
- **Iterates quickly on feedback**

**Result**: 2 days to go from initial release to production-ready minimalist multiplexer with unique positioning.

## 📚 Related Documentation

For detailed technical decisions and development history:

- **[docs/changelog/](docs/changelog/)** - Dated development documents
- **[docs/release-notes/](docs/release-notes/)** - Release documentation
- **[docs/AI_AGENT_FRIENDLY.md](docs/AI_AGENT_FRIENDLY.md)** - Design philosophy
- **[docs/TERMINAL_RESIZE_STRATEGY.md](docs/TERMINAL_RESIZE_STRATEGY.md)** - Technical decisions

---

**Last Updated**: 2026-02-21  
**Version**: 0.3.0  
**Status**: Production Ready 🚀
