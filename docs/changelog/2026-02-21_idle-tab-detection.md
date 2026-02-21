# Idle Tab Detection

**Date**: 2026-02-21  
**Status**: Completed

## Goal

Show a visual marker on tabs and tasks that have had no terminal output for a configurable period, so users can quickly see which sessions are idle across dozens of open tabs.

## Design Decisions

### Why daemon-side tracking?

The client only renders the currently visible tab. Polling all tabs from the client would require one RPC per tab per check — expensive with dozens of tabs open.

The daemon already reads every PTY's output in a background thread, so recording `last_output = Instant::now()` there is zero extra cost.

### Why a single batch RPC?

One `GetIdleStatus` request returns idle seconds for all tabs at once. Cost is constant regardless of how many tabs are open.

### Why 3-second poll interval?

Idle detection doesn't need to be real-time. Polling every 30 frames at 10fps = every 3 seconds is imperceptible to the user and negligible overhead.

## Implementation

### Files Changed

| File | Change |
|------|--------|
| `src/protocol.rs` | Added `ClientMsg::GetIdleStatus`, `ServerMsg::IdleStatus { tabs: Vec<(String, u64)> }` |
| `src/providers/pty_provider.rs` | Added `last_output: Arc<Mutex<Instant>>`, updated in PTY read thread |
| `src/daemon_modules/daemon.rs` | Handles `GetIdleStatus`, returns idle seconds per tab_id |
| `src/client/app.rs` | Added `idle_tabs: HashSet<String>`, `refresh_idle_status()` method |
| `src/main.rs` | Calls `refresh_idle_status()` every 30 frames (~3s) |
| `src/client/ui.rs` | Tab label shows `·` if idle; sidebar task shows `·` if all tabs idle |

### Data Flow

```
PTY read thread
  → has data → last_output = Instant::now()   (per tab, in daemon)

Client main loop (every 30 frames)
  → GetIdleStatus RPC
  → Daemon iterates all tabs, computes now - last_output
  → Returns Vec<(tab_id, idle_secs)>
  → Client updates idle_tabs: HashSet<String>

UI render
  → tab in idle_tabs → show "Terminal 1 ·"
  → all tabs of task in idle_tabs → show "Task 1 ·"
```

## Configuration

Idle threshold is defined as a constant in `src/client/app.rs`:

```rust
const IDLE_THRESHOLD_SECS: u64 = 30;
```

Change this value to adjust when the `·` marker appears.
