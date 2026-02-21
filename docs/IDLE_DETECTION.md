# Mato Idle Tab Detection

Mato automatically detects when a terminal has been quiet for a while and marks it visually, so you can see at a glance which sessions are idle across all your open tabs.

## What It Looks Like

### Idle tab in topbar
```
  Terminal 1    Terminal 2 ·    Terminal 3  
```
The `·` means that tab has had no output for ≥30 seconds.

### Idle task in sidebar
```
   ▶ Task 1
     Task 2 ·
     Task 3
```
The `·` on a task means **all** of its tabs are idle.

## How It Works

- The daemon watches every terminal's output in the background
- Every ~3 seconds, the client asks the daemon: "which tabs have been quiet?"
- Any tab with no output for 30+ seconds gets the `·` marker
- As soon as a tab produces output again, the marker disappears

## What Counts as "Active"

| Situation | Idle? |
|-----------|-------|
| Command is running and printing output | No |
| Shell prompt waiting for input | Yes (no output) |
| Long-running command with no output | Yes |
| Command just finished | Yes (after 30s) |

> **Note**: "Idle" means no terminal *output*, not no user *input*. A tab where you're typing but nothing is printing (e.g. a password prompt) will still be marked idle.

## Default Threshold

30 seconds. To change it, edit `src/client/app.rs`:

```rust
const IDLE_THRESHOLD_SECS: u64 = 30;
```

## Tips

- Use idle markers to spot finished background jobs without switching to each tab
- A task with `·` means you can safely ignore it for now
- No configuration needed — idle detection is always on

## See Also

- [Keyboard Shortcuts](KEYBOARD_SHORTCUTS.md) — Navigate between tabs quickly
- [Persistence Behavior](PERSISTENCE_BEHAVIOR.md) — What survives a restart
