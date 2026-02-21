# Testing Guide

**For AI assistants and developers working on Mato.**

## Current Coverage (2026-02-21)

| Module | Coverage | Notes |
|--------|----------|-------|
| `utils/id.rs` | 100% | ✅ |
| `emulators/vt100_emulator.rs` | 85% | ✅ |
| `daemon_modules/lock.rs` | 89% | ✅ |
| `daemon_modules/pid.rs` | 84% | ✅ |
| `emulators/vte_emulator.rs` | 82% | ✅ |
| `utils/paths.rs` | 81% | ✅ |
| `providers/pty_provider.rs` | 72% | ✅ |
| `client/input.rs` | 49% | ✅ |
| `client/app.rs` | 45% | ✅ |
| `daemon_modules/daemon.rs` | 42% | ✅ via integration tests |
| `config.rs` | 44% | ✅ |
| `client/persistence.rs` | 0% | ⚠️ save/load use fixed path, needs refactor to inject path |
| `client/ui.rs` | 0% | ❌ requires terminal, skip |
| `client/onboarding*.rs` | 0% | ❌ requires terminal, skip |
| `daemon_modules/signals.rs` | 0% | ❌ requires Unix signals |
| `daemon_modules/spawn.rs` | 0% | ❌ forks process, skip |
| `main.rs` | 0% | ❌ entry point, skip |

**Overall: 29% (whole codebase), ~65% (business logic only)**

---

## Test Files

| File | What it tests |
|------|--------------|
| `tests/app_tests.rs` | Task/Tab CRUD, rename, nav, idle filtering, vt100, persistence structs |
| `tests/input_tests.rs` | `handle_key` — all focus transitions, rename buffer, Alt+1-9, sidebar/topbar keys |
| `tests/integration_tests.rs` | Full daemon protocol over real Unix socket (Hello, Spawn, GetScreen, GetIdleStatus) |
| `tests/protocol_tests.rs` | Protocol message serialization |
| `tests/config_tests.rs` | Config load/default/serialization |
| `tests/utils_tests.rs` | ID generation, path helpers |

---

## How to Run

```bash
# All tests
cargo test

# Specific suite
cargo test --test app_tests
cargo test --test input_tests
cargo test --test integration_tests

# Coverage report (requires cargo-llvm-cov)
cargo llvm-cov --summary-only
cargo llvm-cov --text   # per-line detail
```

---

## What's Left to Cover

### Easy wins

**`emulators/vte_emulator.rs` (0% → ~80%)**  
Same pattern as `vt100_emulator.rs` tests. Add to `app_tests.rs`:
```rust
use mato::emulators::VteEmulator;
// test: process text → get_screen shows it
// test: cursor advances after writing
// test: \n moves cursor down, \r resets column
```

**`daemon_modules/pid.rs` (0% → ~80%)**  
Pure file I/O, no daemon needed. Add to a new `tests/daemon_tests.rs`:
```rust
// test: PidFile::create writes current PID to file
// test: PidFile::read parses it back
// test: Drop removes the file
```

**`daemon_modules/lock.rs` (0% → ~60%)**  
```rust
// test: DaemonLock::acquire creates the file
// test: second acquire on same path returns Err (WouldBlock)
// test: Drop removes the lock file
```

**`client/persistence.rs` (0% → ~70%)**  
Needs a temp dir. Use `std::env::temp_dir()`:
```rust
// test: save_state writes valid JSON
// test: load_state reads it back correctly
// test: load_state on missing file returns Err
// test: load_state on corrupt JSON returns Err
```

### Hard / not worth it

| Module | Why skip |
|--------|---------|
| `client/ui.rs` | Requires a real terminal buffer; ratatui rendering tests need a `TestBackend` setup — high effort, low value |
| `client/onboarding*.rs` | Interactive TUI, requires keyboard input simulation |
| `daemon_modules/signals.rs` | Requires sending Unix signals to self; fragile in CI |
| `daemon_modules/spawn.rs` | Forks a process; not suitable for unit tests |
| `main.rs` | Entry point glue code |

---

## Pattern: NullProvider

All tests that need `App` or `Task` use a `NullProvider` to avoid needing a live daemon:

```rust
struct NullProvider;
impl TerminalProvider for NullProvider {
    fn spawn(&mut self, _: u16, _: u16) {}
    fn resize(&mut self, _: u16, _: u16) {}
    fn write(&mut self, _: &[u8]) {}
    fn get_screen(&self, _: u16, _: u16) -> ScreenContent { ScreenContent::default() }
}
```

Copy this into any new test file that needs it.

## Pattern: Integration test daemon

`tests/integration_tests.rs` has a `start_daemon(socket_path)` helper that:
1. Spins up `handle_client` in a background thread with a real `tokio` runtime
2. Waits until the socket accepts connections
3. Returns the shared `tabs: Arc<DashMap<...>>` for inspection

Reuse this helper for any new integration tests.
