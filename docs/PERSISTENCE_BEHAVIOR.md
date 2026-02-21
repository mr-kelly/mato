# Mato Persistence Behavior

## What is Persisted

### ✅ Always Persisted (Survives Client Restart)
- **Task metadata**: Task names, IDs
- **Tab metadata**: Tab names, IDs
- **Active selections**: Active task, active tab per task
- **PTY processes**: Shell processes keep running in daemon

### ⚠️ Persisted Until Daemon Restart
- **PTY screen content**: Terminal output, scrollback buffer
- **PTY state**: Cursor position, colors, attributes

### ❌ Not Persisted (Lost on Daemon Restart)
- **PTY processes**: All shell processes are killed
- **Screen content**: All terminal output is cleared
- **Running commands**: Any commands in progress are terminated

## Behavior Examples

### Scenario 1: Close Client, Reopen Client
**Daemon keeps running**

```
1. Start mato, run: ls -la
2. Close mato (Ctrl+C or 'q')
3. Reopen mato
```

**Result**: ✅
- Task/tab names restored
- Active task/tab restored
- PTY process still running
- Screen content visible (ls -la output still there)

### Scenario 2: Daemon Restarts
**Daemon process killed and restarted**

```
1. Start mato, run: ls -la
2. Kill daemon: pkill -9 mato
3. Reopen mato (auto-starts new daemon)
```

**Result**: ⚠️
- Task/tab names restored
- Active task/tab restored
- PTY processes LOST (new shell spawned)
- Screen content LOST (empty terminal)

### Scenario 3: System Reboot
**Everything cleared**

```
1. Start mato, run: ls -la
2. Reboot system
3. Start mato
```

**Result**: ⚠️
- Task/tab names restored
- Active task/tab restored
- PTY processes LOST
- Screen content LOST

## Why This Design?

### Current Implementation (Phase 1-4)
- **Goal**: Basic persistence for client disconnect/reconnect
- **Trade-off**: Simple implementation, good enough for most use cases
- **Limitation**: Daemon restart loses PTY state

### Future Implementation (Phase 7)
To achieve full persistence like tmux, we need:

1. **PTY State Serialization**
   - Save screen content to disk
   - Save scrollback buffer
   - Save cursor position and attributes

2. **Process Management**
   - Use `criu` (Checkpoint/Restore In Userspace) to save/restore processes
   - Or: Implement custom PTY state capture

3. **Daemon Recovery**
   - Detect daemon crash
   - Restore PTY state from disk
   - Reconnect to saved processes

**Complexity**: High (weeks of work)
**Priority**: Low (current behavior is acceptable for most users)

## Preventing Daemon Restarts

To maximize persistence, keep the daemon running:

### ✅ Good Practices
- Don't kill daemon manually
- Use `mato --status` to check daemon health
- Let daemon run in background indefinitely

### ❌ Avoid
- `pkill mato` (kills daemon)
- `killall mato` (kills daemon)
- System reboots (unavoidable)

### Daemon Auto-Recovery
- Lock file prevents multiple daemons
- Signal handling allows graceful shutdown
- Auto-start on client launch

## Comparison with TMUX

| Feature | TMUX | Mato (Current) | Mato (Future) |
|---------|------|----------------|---------------|
| Task/Tab metadata | ✅ | ✅ | ✅ |
| PTY processes persist | ✅ | ✅ (until daemon restart) | ✅ |
| Screen content persist | ✅ | ✅ (until daemon restart) | ✅ |
| Survive daemon restart | ✅ | ❌ | ✅ (Phase 7) |
| Survive system reboot | ❌ | ❌ | ❌ |

**Note**: Even tmux loses state on system reboot. Full process persistence requires OS-level support (like `criu`).

## Workarounds

### Save Important Output
```bash
# In mato terminal
command | tee output.log
```

### Use tmux Inside mato
```bash
# In mato terminal
tmux new -s mysession
# Now you have double persistence!
```

### Script Your Workflow
```bash
# startup.sh
cd ~/project
npm run dev
```

Then in mato:
```bash
./startup.sh
```

## Summary

**Current behavior is by design and acceptable for most use cases.**

- ✅ Client disconnect/reconnect works perfectly
- ✅ Daemon keeps PTY alive as long as it's running
- ⚠️ Daemon restart loses PTY state (rare event)
- 🎯 Future: Full persistence in Phase 7 (optional enhancement)

**For 99% of use cases, current persistence is sufficient.**
