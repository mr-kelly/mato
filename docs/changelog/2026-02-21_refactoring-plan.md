# Mato Source Code Refactoring Plan

## Current Structure Analysis

```
src/
├── main.rs                    # 300+ lines: CLI entry, daemon spawn, client setup
├── app.rs                     # App state, Focus, EscMode, RenameTarget
├── ui.rs                      # Ratatui rendering
├── input.rs                   # Keyboard input handling
├── id.rs                      # ID generation utility
├── persistence.rs             # Save/load state
├── config.rs                  # TOML config loading
├── protocol.rs                # ClientMsg/ServerMsg enums
├── daemon.rs                  # Daemon server logic
├── terminal_provider.rs       # TerminalProvider trait
├── terminal_emulator.rs       # TerminalEmulator trait
├── providers/
│   ├── mod.rs
│   ├── pty_provider.rs        # Direct PTY with emulator
│   └── daemon_provider.rs     # Client-side daemon communication
└── emulators/
    ├── mod.rs
    ├── vt100_emulator.rs      # Basic ANSI parser
    └── vte_emulator.rs        # Advanced parser
```

## Issues Identified

### 1. **main.rs is too large** (300+ lines)
- Contains daemon spawning logic
- Contains socket path utilities
- Contains status command
- Contains client setup
- **Should be**: Thin entry point that delegates to modules

### 2. **Daemon-related code scattered**
- `main.rs`: daemon spawning, socket path
- `daemon.rs`: daemon server
- `providers/daemon_provider.rs`: client-side
- **Should be**: Organized under `daemon/` module

### 3. **Missing utility modules**
- Socket path logic duplicated
- No centralized error types
- No logging setup module

### 4. **No separation of concerns**
- Client and daemon code mixed in main.rs
- CLI parsing done manually

---

## Proposed Structure

```
src/
├── main.rs                    # Thin entry point (~50 lines)
├── cli.rs                     # NEW: CLI argument parsing
├── client/                    # NEW: Client-specific code
│   ├── mod.rs
│   ├── app.rs                 # Move from root
│   ├── ui.rs                  # Move from root
│   ├── input.rs               # Move from root
│   └── persistence.rs         # Move from root
├── daemon/                    # NEW: Daemon-specific code
│   ├── mod.rs
│   ├── server.rs              # Rename from daemon.rs
│   ├── spawn.rs               # NEW: Daemon spawning logic
│   ├── status.rs              # NEW: Status command
│   └── lock.rs                # NEW: Lock file (Phase 4A)
├── providers/
│   ├── mod.rs
│   ├── pty_provider.rs
│   └── daemon_provider.rs
├── emulators/
│   ├── mod.rs
│   ├── vt100_emulator.rs
│   └── vte_emulator.rs
├── protocol.rs                # Keep at root (shared)
├── terminal_provider.rs       # Keep at root (trait)
├── terminal_emulator.rs       # Keep at root (trait)
├── config.rs                  # Keep at root (shared)
├── utils/                     # NEW: Utilities
│   ├── mod.rs
│   ├── paths.rs               # Socket/log/config paths
│   ├── id.rs                  # Move from root
│   └── logging.rs             # NEW: Logging setup
└── error.rs                   # NEW: Custom error types
```

---

## Refactoring Steps

### Step 1: Create New Modules (No Breaking Changes)

```bash
# Create new directories
mkdir -p src/client src/daemon src/utils

# Create module files
touch src/cli.rs
touch src/error.rs
touch src/client/mod.rs
touch src/daemon/mod.rs
touch src/utils/mod.rs
touch src/utils/paths.rs
touch src/utils/logging.rs
touch src/daemon/spawn.rs
touch src/daemon/status.rs
```

### Step 2: Extract Utilities

**src/utils/paths.rs**:
```rust
use std::path::PathBuf;

pub fn get_socket_path() -> PathBuf {
    let state_dir = dirs::state_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("mato");
    std::fs::create_dir_all(&state_dir).ok();
    state_dir.join("daemon.sock")
}

pub fn get_daemon_log_path() -> PathBuf {
    get_socket_path().with_file_name("daemon.log")
}

pub fn get_client_log_path() -> PathBuf {
    get_socket_path().with_file_name("client.log")
}

pub fn get_pid_path() -> PathBuf {
    get_socket_path().with_file_name("daemon.pid")
}

pub fn get_lock_path() -> PathBuf {
    get_socket_path().with_file_name("daemon.lock")
}
```

**src/utils/logging.rs**:
```rust
use std::path::Path;

pub fn setup_daemon_logging(log_path: &Path) -> anyhow::Result<()> {
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    
    tracing_subscriber::fmt()
        .with_writer(log_file)
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        .init();
    
    Ok(())
}

pub fn setup_client_logging(log_path: &Path) -> anyhow::Result<()> {
    // Similar to daemon
}
```

### Step 3: Extract CLI Parsing

**src/cli.rs**:
```rust
pub enum Command {
    Client,
    Daemon { foreground: bool },
    Status,
}

pub fn parse_args() -> Command {
    let args: Vec<_> = std::env::args().collect();
    
    if args.contains(&"--daemon".to_string()) {
        let foreground = args.contains(&"--foreground".to_string());
        Command::Daemon { foreground }
    } else if args.contains(&"--status".to_string()) {
        Command::Status
    } else {
        Command::Client
    }
}
```

### Step 4: Extract Daemon Spawning

**src/daemon/spawn.rs**:
```rust
use crate::utils::paths;
use std::process::Command;

pub fn ensure_daemon_running() -> anyhow::Result<()> {
    let socket_path = paths::get_socket_path();
    
    // Check if daemon is running
    if socket_path.exists() {
        if can_connect(&socket_path) {
            return Ok(());
        }
        // Stale socket
        std::fs::remove_file(&socket_path)?;
    }
    
    // Spawn daemon
    spawn_daemon()?;
    
    // Wait for socket
    wait_for_socket(&socket_path)?;
    
    Ok(())
}

fn spawn_daemon() -> anyhow::Result<()> {
    Command::new(std::env::current_exe()?)
        .arg("--daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    Ok(())
}
```

### Step 5: Extract Status Command

**src/daemon/status.rs**:
```rust
use crate::utils::paths;
use crate::protocol::{ClientMsg, ServerMsg};

pub fn show_status() -> anyhow::Result<()> {
    let socket_path = paths::get_socket_path();
    
    if !socket_path.exists() {
        println!("❌ Daemon not running");
        return Ok(());
    }
    
    // Connect and query status
    // ... (move logic from main.rs)
    
    Ok(())
}
```

### Step 6: Move Client Code

```bash
# Move files
git mv src/app.rs src/client/app.rs
git mv src/ui.rs src/client/ui.rs
git mv src/input.rs src/client/input.rs
git mv src/persistence.rs src/client/persistence.rs

# Update imports in moved files
# Update src/client/mod.rs
```

### Step 7: Rename Daemon Module

```bash
git mv src/daemon.rs src/daemon/server.rs
# Update src/daemon/mod.rs
```

### Step 8: Simplify main.rs

**New src/main.rs** (~50 lines):
```rust
mod cli;
mod client;
mod daemon;
mod providers;
mod emulators;
mod protocol;
mod terminal_provider;
mod terminal_emulator;
mod config;
mod utils;
mod error;

use cli::Command;

fn main() -> anyhow::Result<()> {
    match cli::parse_args() {
        Command::Client => client::run(),
        Command::Daemon { foreground } => daemon::run(foreground),
        Command::Status => daemon::show_status(),
    }
}
```

---

## Benefits

### 1. **Clarity**
- Clear separation: client vs daemon vs shared
- Easy to find code: "Where's the status command?" → `daemon/status.rs`

### 2. **Maintainability**
- Smaller files (< 200 lines each)
- Single responsibility per module
- Easier to test

### 3. **Scalability**
- Easy to add new commands (just add to `cli.rs`)
- Easy to add new daemon features (just add to `daemon/`)
- Easy to add new client features (just add to `client/`)

### 4. **Preparation for Phase 4**
- `daemon/lock.rs` ready for lock file
- `daemon/signals.rs` ready for signal handling
- `daemon/pid.rs` ready for PID file

---

## Testing Strategy

### 1. Refactor in Small Steps
- Each step should compile
- Run tests after each step
- Commit after each successful step

### 2. No Behavior Changes
- Refactoring should not change functionality
- All existing features should work

### 3. Incremental Migration
- Keep old code working during migration
- Use `#[deprecated]` for old functions
- Remove old code only after migration complete

---

## Estimated Effort

- **Step 1-2**: 30 minutes (create modules, extract utilities)
- **Step 3-5**: 1 hour (extract CLI, daemon spawn, status)
- **Step 6-7**: 1 hour (move client code, rename daemon)
- **Step 8**: 30 minutes (simplify main.rs)
- **Testing**: 1 hour (verify everything works)

**Total: ~4 hours**

---

## Success Criteria

- [ ] main.rs < 100 lines
- [ ] All modules < 300 lines
- [ ] Clear separation: client/ vs daemon/ vs shared
- [ ] All tests pass
- [ ] No behavior changes
- [ ] Easier to navigate codebase
- [ ] Ready for Phase 4 implementation
