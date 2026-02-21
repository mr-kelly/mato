# Terminal State Persistence - Future Design

## Current Limitations

**What persists across client restart**: ✅
- PTY process (bash continues running in daemon)
- Screen content (emulator buffer in memory)
- Running commands continue executing

**What does NOT persist across daemon restart**: ❌
- PTY process (killed when daemon exits)
- Screen content (lost with daemon process)
- All terminal state

## Storage Locations

### Current (v0.2.0)

```
~/.config/mato/state.json
{
  "tasks": [
    {
      "id": "task-1",
      "name": "Development",
      "tabs": [
        {
          "id": "tab-abc123",  // ← Only ID and name!
          "name": "Terminal 1"
        }
      ]
    }
  ]
}
```

**Missing**: Terminal content, cursor position, colors, etc.

### Future (Phase 10)

```
~/.local/state/mato/
├── daemon.sock
├── daemon.pid
├── daemon.log
└── tabs/                    ← NEW!
    ├── tab-abc123.json      ← Terminal state
    ├── tab-def456.json
    └── tab-ghi789.json
```

Each tab state file:
```json
{
  "tab_id": "tab-abc123",
  "size": { "rows": 24, "cols": 80 },
  "cursor": { "row": 5, "col": 10 },
  "screen": [
    {
      "cells": [
        { "ch": "$", "fg": "green", "bg": null },
        { "ch": " ", "fg": null, "bg": null }
      ]
    }
  ],
  "scrollback": [],  // Phase 9
  "last_updated": "2026-02-21T20:00:00Z"
}
```

## Implementation Plan

### Phase 9: Scrollback Buffer (Prerequisite)

Before we can persist state, we need to store more than just the current screen:

```rust
pub struct TerminalState {
    screen: Vec<Line>,           // Current visible screen
    scrollback: VecDeque<Line>,  // Historical lines (1000+)
    cursor: (u16, u16),
    size: (u16, u16),
}
```

### Phase 10: State Serialization

#### 1. Serialize on Interval

```rust
// In daemon, every 30 seconds
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        save_all_tab_states(&tabs).await;
    }
});
```

#### 2. Serialize on Shutdown

```rust
impl Daemon {
    fn shutdown(&self) {
        tracing::info!("Saving terminal states before shutdown");
        for entry in self.tabs.iter() {
            let (tab_id, provider) = entry.pair();
            save_tab_state(tab_id, provider);
        }
        // ... rest of shutdown
    }
}
```

#### 3. Restore on Startup

```rust
impl Daemon {
    pub fn new() -> Self {
        let daemon = Self { /* ... */ };
        
        // Restore saved states
        if let Ok(saved_tabs) = load_saved_tab_states() {
            for (tab_id, state) in saved_tabs {
                // Create new PTY with restored screen content
                let mut provider = PtyProvider::new();
                provider.spawn(state.size.0, state.size.1);
                provider.restore_screen_content(state);
                daemon.tabs.insert(tab_id, Arc::new(Mutex::new(provider)));
            }
        }
        
        daemon
    }
}
```

## Limitations Even with Persistence

### What CAN be restored ✅
- Screen content (text, colors, cursor position)
- Scrollback history
- Terminal size

### What CANNOT be restored ❌
- **Running processes** - Bash process is killed, can't be resumed
- **Command history** - Bash's internal history is lost
- **Environment variables** - Process-specific state
- **Working directory** - Unless we track it separately

### Example Scenario

**Before daemon restart:**
```
$ cd /home/user/project
$ npm run dev
[Server running on port 3000...]
```

**After daemon restart with state persistence:**
```
$ cd /home/user/project
$ npm run dev
[Server running on port 3000...]  ← Screen content restored
                                   ← But server is NOT running!
                                   ← Just showing old text
```

**This is misleading!** User sees "Server running" but it's not actually running.

## Better Approach: Session Scripts

Instead of trying to restore process state, save the **commands to recreate the state**:

```json
{
  "tab_id": "tab-abc123",
  "restore_script": [
    "cd /home/user/project",
    "npm run dev"
  ],
  "auto_restore": false  // User must opt-in
}
```

On daemon restart:
1. Show saved screen content (read-only)
2. Offer to run restore script
3. User confirms → commands are executed

## Recommendation

### For v0.2.0 (Current)
- ✅ Document that daemon restart loses state
- ✅ Recommend keeping daemon running
- ✅ Use systemd/supervisor for daemon auto-restart

### For Phase 9
- Add scrollback buffer
- Add copy mode
- Foundation for state persistence

### For Phase 10
- Implement screen content serialization
- Add restore scripts (opt-in)
- Clear UX: "This is saved content, not live"

### For Phase 11
- Integrate with tmux resurrect patterns
- Save/restore working directory
- Smart detection of restorable commands

## Conclusion

**Current behavior is correct**: 
- Client restart → content persists ✅
- Daemon restart → content lost ❌ (by design)

**Future enhancement**: 
- Save screen content to disk
- But don't try to restore process state (impossible)
- Instead, offer to re-run commands

---

**Status**: Design document for future implementation  
**Priority**: Phase 10 (after scrollback buffer)
