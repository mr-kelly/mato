# Terminal Persistence Guide

**Quick Answer**: Terminal content survives client restart, but not daemon restart.

## 🎯 What Persists

### ✅ Survives Client Restart

When you close and reopen Mato:

| What | Status | Details |
|------|--------|---------|
| **PTY Process** | ✅ Keeps running | Bash continues in daemon |
| **Screen Content** | ✅ Preserved | Current visible screen |
| **Running Commands** | ✅ Continue | Long-running processes keep going |
| **Task/Tab Structure** | ✅ Restored | Saved to `state.json` |

**Example**:
```bash
# Start a long-running command
npm run dev

# Close Mato (Ctrl+Q)
# Reopen Mato

# Your npm process is still running! ✅
```

### ❌ Lost on Daemon Restart

When daemon is killed/restarted:

| What | Status | Details |
|------|--------|---------|
| **PTY Process** | ❌ Killed | All bash processes terminated |
| **Screen Content** | ❌ Lost | Terminal output cleared |
| **Running Commands** | ❌ Stopped | All processes terminated |
| **Task/Tab Structure** | ✅ Restored | Still saved in `state.json` |

### ⚠️ Current Limitations

| Limitation | Impact | Workaround |
|------------|--------|------------|
| **No scrollback** | Can't scroll up | Use `script` or `tmux` inside Mato |
| **Resize clears screen** | ❌ Fixed in v0.3.0! | Content now preserved |
| **Daemon restart** | Loses all state | Keep daemon running |

## 🔧 How It Works

### Architecture

```
Client (TUI) ←→ Unix Socket ←→ Daemon (PTY Manager)
                                    ↓
                              PTY Processes
                              (bash, vim, etc.)
```

**Key Points**:
- PTY processes run in **daemon**, not client
- Client just displays what daemon sends
- Closing client doesn't affect PTY processes
- Killing daemon terminates all PTYs

### Persistence Mechanism

1. **Task/Tab Metadata** → `~/.config/mato/state.json`
   ```json
   {
     "tasks": [
       {
         "id": "task-1",
         "name": "Development",
         "tabs": [
           {"id": "tab-abc", "name": "Terminal 1"}
         ]
       }
     ]
   }
   ```

2. **PTY State** → Daemon memory (not saved to disk)
   - Screen buffer
   - Cursor position
   - Running processes

3. **On Client Restart**:
   - Load `state.json` → Get tab IDs
   - Connect to daemon → Request screen content
   - Daemon returns current PTY screen
   - Display restored! ✅

## 📋 Best Practices

### Keep Daemon Running

```bash
# Check daemon status
mato --status

# Daemon should show:
# ✓ Running
# Uptime: 2 days, 5 hours
```

### For Important Work

1. **Use tmux inside Mato** - Additional persistence layer
2. **Save output to files** - `command > output.log`
3. **Use systemd** - Auto-restart daemon on crash

### Daemon Management

```bash
# Start daemon manually
mato --daemon

# Check if running
ps aux | grep mato

# Graceful shutdown
kill -TERM $(cat ~/.local/state/mato/daemon.pid)
```

## 🐛 Troubleshooting

### Content Disappeared After Resize

**Fixed in v0.3.0!** ✅

If you're on older version:
- Upgrade to v0.3.0
- Daemon no longer resizes PTY on window resize
- Content always preserved

### Content Lost After Client Restart

**Check**:
1. Is daemon still running? `mato --status`
2. Are you using the same tab ID?
3. Check daemon logs: `tail ~/.local/state/mato/daemon.log`

**Common causes**:
- Daemon was restarted (kills all PTYs)
- Different tab ID (new PTY created)
- PTY process crashed

### Daemon Keeps Dying

**Solutions**:
1. Check logs: `~/.local/state/mato/daemon.log`
2. Use systemd for auto-restart
3. Report bug with logs

## 🔮 Future Improvements

See [FUTURE_STATE_PERSISTENCE.md](todos/FUTURE_STATE_PERSISTENCE.md) for planned features:

- **Phase 9**: Scrollback buffer
- **Phase 10**: Serialize PTY state to disk
- **Phase 11**: Smart resize (preserve content)

## 📚 Related Documentation

- **[TERMINAL_RESIZE_STRATEGY.md](TERMINAL_RESIZE_STRATEGY.md)** - How resize works
- **[KEYBOARD_SHORTCUTS.md](KEYBOARD_SHORTCUTS.md)** - All shortcuts
- **[ACTIVITY_INDICATORS.md](ACTIVITY_INDICATORS.md)** - Real-time activity spinners

---

**TL;DR**: Close client anytime, content stays. Don't restart daemon unless necessary.

### 1. Resize Clears Screen

When the terminal size changes, the emulator may need to recreate its internal buffer, which can clear the screen content. This is a limitation of the underlying `vt100` parser library.

**Workaround**: Avoid resizing the terminal window if you want to preserve content.

### 2. No Scrollback Buffer

Only the current visible screen is preserved. If you scroll up in bash, that history is not saved by MATO.

**Future**: Phase 9 will add scrollback buffer support (copy mode).

### 3. Daemon Restart Loses State

If the daemon process is killed or crashes, all PTY state is lost.

**Future**: Consider implementing PTY state serialization for true persistence.

## Best Practices

### To Preserve Terminal Content

1. **Don't resize the terminal** - Keep the same window size
2. **Use tmux/screen inside MATO** - For additional persistence layer
3. **Save important output** - Redirect to files for critical data

### To Avoid Content Loss

1. **Keep daemon running** - Don't kill the daemon process
2. **Use consistent terminal size** - Set a fixed window size
3. **Monitor daemon health** - Check `mato --status` regularly

## Technical Details

### Architecture

```
Client (TUI)
    ↓ Unix Socket
Daemon
    ↓ PTY
Bash Process
    ↓ Terminal Emulator (vt100/vte)
Screen Buffer (rows × cols)
```

### Resize Flow

```
1. Client detects window resize
2. Client sends Resize message to daemon
3. Daemon resizes PTY master
4. Daemon resizes emulator buffer
5. Emulator may recreate buffer (clears content)
6. Bash receives SIGWINCH
7. Bash redraws prompt
```

### Reconnection Flow

```
1. Client starts, loads state.json (tab IDs)
2. Client sends Spawn message for each tab
3. Daemon checks if PTY exists
   - If exists: Skip spawn, preserve content ✅
   - If not exists: Create new PTY
4. Client calls get_screen() to fetch content
5. Content is displayed
```

## Future Improvements

### Phase 9: Copy Mode & Scrollback
- Implement scrollback buffer (1000+ lines)
- Add copy mode with vim-style navigation
- Search in scrollback history

### Phase 10: State Serialization
- Serialize emulator state to disk
- Restore state on daemon restart
- Survive daemon crashes

### Phase 11: Smart Resize
- Preserve content during resize
- Reflow text intelligently
- Use better terminal emulator library

## Troubleshooting

### Content Disappears on Reconnect

**Cause**: Daemon was restarted or resize was triggered

**Solution**: 
1. Check daemon uptime: `mato --status`
2. Avoid resizing terminal window
3. Keep daemon running continuously

### Screen Flickers/Clears

**Cause**: Unnecessary resize calls

**Solution**: Fixed in v0.2.0 - resize only when size actually changes

### Idle Tabs Show Old Content

**Cause**: Bash may have cleared screen or exited

**Solution**: This is expected behavior - the PTY reflects actual bash state

---

**Last Updated**: 2026-02-21  
**Version**: 0.2.0
