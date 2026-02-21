# Phase 4: Daemon Improvements (Inspired by TMUX)

**Goal**: Reach 100% TMUX daemon/client capability parity
**Current**: 76% complete
**Target**: 100% complete

---

## Phase 4A: Critical Fixes (Priority 1)

**Goal**: Fix critical reliability and security issues
**Estimated effort**: 1-2 days
**Completion**: 76% → 85%

### Task 1: Lock File Mechanism ⚠️ CRITICAL

**Problem**: Race condition when multiple clients start daemon simultaneously

**Implementation**:
```rust
// src/daemon_lock.rs
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use nix::fcntl::{flock, FlockArg};

pub struct DaemonLock {
    file: File,
    path: PathBuf,
}

impl DaemonLock {
    pub fn acquire() -> Result<Self> {
        let path = get_lock_path(); // ~/.local/state/mato/daemon.lock
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)?;
        
        // Try to acquire exclusive lock (non-blocking)
        flock(file.as_raw_fd(), FlockArg::LockExclusiveNonblock)
            .map_err(|_| Error::AlreadyLocked)?;
        
        // Write PID
        write!(file, "{}", std::process::id())?;
        
        Ok(Self { file, path })
    }
}

impl Drop for DaemonLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
```

**Usage**:
```rust
// In ensure_daemon_running()
let _lock = DaemonLock::acquire()?;
// Spawn daemon
// Lock released when _lock drops
```

**Dependencies**: Add `nix = "0.27"` to Cargo.toml

---

### Task 2: Signal Handling ⚠️ CRITICAL

**Problem**: Daemon cannot be gracefully shut down or reloaded

**Implementation**:
```rust
// src/daemon_signals.rs
use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;
use tokio::sync::broadcast;

pub enum DaemonSignal {
    Shutdown,
    Reload,
}

pub fn setup_signal_handlers() -> broadcast::Receiver<DaemonSignal> {
    let (tx, rx) = broadcast::channel(16);
    
    let mut signals = Signals::new(&[SIGTERM, SIGINT, SIGHUP])
        .expect("Failed to register signals");
    
    tokio::spawn(async move {
        for sig in signals.forever() {
            match sig {
                SIGTERM | SIGINT => {
                    tracing::info!("Received shutdown signal");
                    let _ = tx.send(DaemonSignal::Shutdown);
                    break;
                }
                SIGHUP => {
                    tracing::info!("Received SIGHUP, reloading config");
                    let _ = tx.send(DaemonSignal::Reload);
                }
                _ => {}
            }
        }
    });
    
    rx
}
```

**Usage**:
```rust
// In daemon::run()
let mut signals = setup_signal_handlers();

tokio::select! {
    _ = listener.accept() => { /* handle client */ }
    sig = signals.recv() => {
        match sig {
            Ok(DaemonSignal::Shutdown) => {
                tracing::info!("Shutting down gracefully");
                break;
            }
            Ok(DaemonSignal::Reload) => {
                // Reload config
            }
            _ => {}
        }
    }
}
```

**Dependencies**: Add `signal-hook = "0.3"` to Cargo.toml

---

### Task 3: Socket Permissions 🔒 SECURITY

**Problem**: Socket file has default permissions (potential security issue)

**Implementation**:
```rust
// In daemon::run() after bind()
use std::os::unix::fs::PermissionsExt;

let listener = UnixListener::bind(socket_path)?;

// Set socket to owner-only (0700)
let perms = std::fs::Permissions::from_mode(0o700);
std::fs::set_permissions(socket_path, perms)?;

tracing::info!("Socket permissions set to 0700");
```

**No dependencies needed**

---

## Phase 4B: Reliability Improvements (Priority 2)

**Goal**: Improve daemon reliability and management
**Estimated effort**: 2-3 days
**Completion**: 85% → 92%

### Task 4: Graceful Shutdown

**Implementation**:
```rust
// In daemon::run()
impl Daemon {
    pub async fn shutdown(&self) {
        tracing::info!("Starting graceful shutdown");
        
        // Close all PTY sessions
        for entry in self.tabs.iter() {
            let (tab_id, _) = entry.pair();
            tracing::info!("Closing tab {}", tab_id);
        }
        
        // Clear tabs
        self.tabs.clear();
        
        tracing::info!("Graceful shutdown complete");
    }
}
```

---

### Task 5: PID File

**Implementation**:
```rust
// src/daemon_pid.rs
pub struct PidFile {
    path: PathBuf,
}

impl PidFile {
    pub fn create() -> Result<Self> {
        let path = get_pid_path(); // ~/.local/state/mato/daemon.pid
        let pid = std::process::id();
        std::fs::write(&path, pid.to_string())?;
        Ok(Self { path })
    }
    
    pub fn read() -> Option<u32> {
        let path = get_pid_path();
        std::fs::read_to_string(path)
            .ok()?
            .trim()
            .parse()
            .ok()
    }
}

impl Drop for PidFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
```

---

### Task 6: Verify Event Loop After Fork

**Implementation**:
```rust
// In run_daemon() after fork
// Recreate tokio runtime in child process
let rt = tokio::runtime::Runtime::new()?;
rt.block_on(async {
    daemon.run(&socket_path).await
})?;
```

**Test**: Verify no deadlocks or panics after fork

---

## Phase 4C: Polish & Features (Priority 3)

**Goal**: Reach feature parity with TMUX
**Estimated effort**: 3-5 days
**Completion**: 92% → 100%

### Task 7: Multiple Clients Support

**Problem**: Only one client can connect at a time

**Implementation**:
- Broadcast PTY output to all connected clients
- Use `tokio::sync::broadcast` channel
- Each client gets a receiver

---

### Task 8: Config Reload on SIGHUP

**Implementation**:
```rust
// On SIGHUP signal
let new_config = Config::load();
// Apply new config to running daemon
// (e.g., change emulator for new tabs)
```

---

### Task 9: Enhanced Status Command

**Implementation**:
```rust
// mato --status output
Mato Daemon Status
==================
Status: ✓ Running
PID: 12345
Started: 2026-02-21 16:00:00
Uptime: 2h 15m
Socket: /home/user/.local/state/mato/daemon.sock
Config: /home/user/.config/mato/config.toml

Active Sessions:
  Task 1: 3 tabs
  Task 2: 5 tabs
  Total: 8 tabs

Memory: 45 MB
CPU: 0.5%
```

---

### Task 10: Better Error Handling

**Implementation**:
- Add custom error types
- Better error messages
- Retry logic for transient failures
- Log errors to daemon.log

---

## Testing Checklist

### Phase 4A Tests
- [ ] Multiple clients starting daemon simultaneously (lock file)
- [ ] Send SIGTERM to daemon (graceful shutdown)
- [ ] Send SIGHUP to daemon (config reload)
- [ ] Check socket permissions (should be 0700)

### Phase 4B Tests
- [ ] Daemon shutdown cleans up all resources
- [ ] PID file created and removed correctly
- [ ] Tokio runtime works after fork (no deadlocks)

### Phase 4C Tests
- [ ] Multiple clients can connect and share view
- [ ] Config reload works without restart
- [ ] Status command shows detailed info
- [ ] Error messages are helpful

---

## Dependencies to Add

```toml
[dependencies]
# Existing...
nix = "0.27"           # For flock() in lock file
signal-hook = "0.3"    # For signal handling
```

---

## File Structure After Phase 4

```
src/
├── daemon_lock.rs      # NEW: Lock file management
├── daemon_signals.rs   # NEW: Signal handling
├── daemon_pid.rs       # NEW: PID file management
├── daemon.rs           # MODIFIED: Add shutdown logic
├── main.rs             # MODIFIED: Use lock file
└── ...
```

---

## Success Metrics

- [ ] No race conditions when starting daemon
- [ ] Daemon responds to SIGTERM/SIGINT/SIGHUP
- [ ] Socket has correct permissions (0700)
- [ ] Graceful shutdown cleans up all resources
- [ ] PID file tracks daemon process
- [ ] Multiple clients can connect
- [ ] Config reload works
- [ ] All tests pass

**Target**: 100% TMUX daemon/client capability parity
