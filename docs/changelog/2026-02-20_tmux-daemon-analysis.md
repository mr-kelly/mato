# TMUX Daemon Architecture Analysis

## Reference
Source: `/home/kelly/Documents/refs/tmux/`

## Key Findings from TMUX Implementation

### 1. Fork and Daemon Process (`proc.c:359`)

**TMUX Approach:**
```c
pid_t proc_fork_and_daemon(int *fd) {
    pid_t pid;
    int pair[2];
    
    // Create socketpair for parent-child communication
    socketpair(AF_UNIX, SOCK_STREAM, PF_UNSPEC, pair);
    
    switch (pid = fork()) {
    case 0:  // Child
        close(pair[0]);
        *fd = pair[1];
        daemon(1, 0);  // Daemonize
        return (0);
    default: // Parent
        close(pair[1]);
        *fd = pair[0];
        return (pid);
    }
}
```

**Key Points:**
- Uses `socketpair()` for bidirectional communication between parent and child
- Parent gets one end, child gets the other
- Child calls `daemon(1, 0)` to properly daemonize
- Returns different values to parent (pid) and child (0)

**Mato Current:**
- ✅ We use Unix socket (file-based)
- ❌ We don't use socketpair (could be more reliable)
- ✅ We fork with `libc::fork()` + `libc::setsid()`
- ⚠️ Could improve: Use `daemon()` syscall instead of manual fork

### 2. Server Start Sequence (`server.c:176`)

**TMUX Approach:**
```c
int server_start(...) {
    sigset_t set, oldset;
    
    // 1. Block all signals during fork
    sigfillset(&set);
    sigprocmask(SIG_BLOCK, &set, &oldset);
    
    // 2. Fork and daemon
    if (~flags & CLIENT_NOFORK) {
        if (proc_fork_and_daemon(&fd) != 0) {
            sigprocmask(SIG_SETMASK, &oldset, NULL);
            return (fd);  // Parent returns
        }
    }
    
    // 3. Clear signals in child
    proc_clear_signals(client, 0);
    
    // 4. Reinit event loop
    event_reinit(base);
    
    // 5. Set up signal handlers
    proc_set_signals(server_proc, server_signal);
    sigprocmask(SIG_SETMASK, &oldset, NULL);
    
    // 6. Create server socket
    server_fd = server_create_socket(flags, &cause);
    server_update_socket();
    
    // 7. Start event loop
    evtimer_set(&server_ev_tidy, server_tidy_event, NULL);
    evtimer_add(&server_ev_tidy, &tv);
}
```

**Key Improvements for Mato:**

1. **Signal Handling** ⚠️
   - TMUX blocks all signals during fork
   - Clears signal handlers in child
   - Sets up proper signal handlers after fork
   - **Mato TODO**: Add signal handling (SIGTERM, SIGHUP, SIGINT)

2. **Event Loop Reinit** ⚠️
   - TMUX calls `event_reinit(base)` after fork
   - Important for libevent/tokio after fork
   - **Mato TODO**: Verify tokio runtime works correctly after fork

3. **Socket Creation** ✅
   - TMUX creates socket after fork (in child)
   - **Mato**: Already doing this correctly

4. **Lock File Management** ⚠️
   - TMUX uses lock file to prevent multiple servers
   - Removes lock file after successful start
   - **Mato TODO**: Add lock file mechanism

### 3. Client Connection (`client.c:73`)

**TMUX Approach:**
```c
// Get server create lock
fd = server_start(client_proc, flags, base, lockfd, lockfile);

// If lock held, server start is happening in another process
// Wait and retry connection
```

**Key Points:**
- Uses lock file to coordinate multiple clients starting server
- Only one client can start the server
- Others wait and retry connection

**Mato Current:**
- ✅ We check if socket exists
- ✅ We retry connection with timeout
- ❌ No lock file (race condition possible)

### 4. Socket Management

**TMUX:**
- Creates socket in `$TMUX_TMPDIR` or `/tmp/tmux-$UID/`
- Socket path: `/tmp/tmux-$UID/default`
- Removes stale sockets on startup
- Updates socket permissions

**Mato:**
- ✅ Socket in `~/.local/state/mato/daemon.sock`
- ✅ Removes stale sockets
- ⚠️ Could improve: Check socket permissions

---

## Recommended Improvements for Mato

### Priority 1: Critical

1. **Add Lock File Mechanism**
   - Prevent race condition when multiple clients start daemon
   - Use flock() or similar
   - Location: `~/.local/state/mato/daemon.lock`

2. **Signal Handling**
   - Handle SIGTERM for graceful shutdown
   - Handle SIGHUP to reload config
   - Block signals during fork

### Priority 2: Important

3. **Use daemon() syscall**
   - Replace manual fork + setsid with `daemon(1, 0)`
   - More reliable and standard

4. **Socket Permissions**
   - Set socket to 0700 (owner only)
   - Security improvement

5. **Event Loop Verification**
   - Verify tokio runtime works after fork
   - May need to recreate runtime in child

### Priority 3: Nice to Have

6. **Socketpair Communication**
   - Use socketpair instead of file-based socket
   - More reliable for parent-child communication
   - Keep file socket for client connections

7. **PID File**
   - Write daemon PID to file
   - Easier to manage daemon process
   - Location: `~/.local/state/mato/daemon.pid`

8. **Cleanup on Exit**
   - Remove socket file on clean exit
   - Remove PID file
   - Close all PTYs gracefully

---

## Implementation Plan

### Phase 4: Daemon Improvements (from TMUX)

**Task 1: Lock File**
```rust
// src/daemon_lock.rs
pub struct DaemonLock {
    file: File,
    path: PathBuf,
}

impl DaemonLock {
    pub fn acquire() -> Result<Self> {
        let path = get_lock_path();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)?;
        
        // Try to acquire exclusive lock (non-blocking)
        flock(file.as_raw_fd(), FlockArg::LockExclusiveNonblock)?;
        
        Ok(Self { file, path })
    }
}

impl Drop for DaemonLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
```

**Task 2: Signal Handling**
```rust
// In daemon startup
use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;

let mut signals = Signals::new(&[SIGTERM, SIGINT, SIGHUP])?;
tokio::spawn(async move {
    for sig in signals.forever() {
        match sig {
            SIGTERM | SIGINT => {
                tracing::info!("Received shutdown signal");
                // Graceful shutdown
                break;
            }
            SIGHUP => {
                tracing::info!("Received SIGHUP, reloading config");
                // Reload config
            }
            _ => {}
        }
    }
});
```

**Task 3: Socket Permissions**
```rust
use std::os::unix::fs::PermissionsExt;

// After creating socket
let perms = std::fs::Permissions::from_mode(0o700);
std::fs::set_permissions(&socket_path, perms)?;
```

---

## References

- TMUX source: `/home/kelly/Documents/refs/tmux/`
- Key files:
  - `server.c` - Server startup
  - `proc.c` - Process management
  - `client.c` - Client connection
