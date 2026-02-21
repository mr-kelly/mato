# Activity Indicators

Mato shows real-time spinners on terminals that are actively producing output, so you can see at a glance which tasks are working - perfect for monitoring AI agents and long-running processes.

## 🎯 What It Looks Like

### Active tab in topbar
```
  Terminal 1 ⠋    Terminal 2    Terminal 3 ⠴  
             ↑                              ↑
         Working!                      Working!
```
The animated spinner (⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏) means that tab has produced output in the last 2 seconds.

### Active task in sidebar
```
   ▶ Development ⠋
     Testing
     Production ⠴
```
The spinner on a task means **at least one** of its tabs is active.

## 🎬 How It Works

### Detection Logic

- The daemon tracks every terminal's last output timestamp
- Every frame (~80ms), the client asks: "which tabs are active?"
- Any tab with output in the last **2 seconds** gets a spinner
- As soon as 2 seconds pass with no output, the spinner disappears

### Animation

- **10 frames**: ⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
- **Frame rate**: 80ms per frame = 12.5 FPS
- **Smooth loop**: Seamless animation cycle

### Performance

| State | Polling Rate | FPS | CPU Usage |
|-------|--------------|-----|-----------|
| **Has active tabs** | 80ms | 12.5 | Medium |
| **All idle** | 200ms | 5 | Minimal |

**Smart adaptation**: When all tabs are idle, Mato automatically reduces polling rate to save CPU.

## 🎯 What Counts as "Active"

| Situation | Active? | Spinner? |
|-----------|---------|----------|
| Command just started | ✅ Yes | ✅ Shows |
| Command producing output | ✅ Yes | ✅ Shows |
| Command finished 1 second ago | ✅ Yes | ✅ Shows |
| Command finished 3 seconds ago | ❌ No | ❌ Hidden |
| Long-running silent command | ❌ No | ❌ Hidden |
| Interactive prompt (waiting for input) | ❌ No | ❌ Hidden |

**Key insight**: Spinners show **output activity**, not process existence.

## 💡 Use Cases

### 🤖 Monitoring AI Agents

```
┌─────────────────────────────────────────────────────────┐
│  Claude Agent ⠋    Codex CLI    GitHub Copilot ⠴       │
├─────────────────────────────────────────────────────────┤
│ ▶ AI Agents ⠋    │  $ claude "Build REST API"          │
│   Development    │  ⠋ Analyzing requirements...         │
│   Testing        │  ⠋ Generating code...                │
└─────────────────────────────────────────────────────────┘
```

**Benefits:**
- ✅ Know which agent is working
- ✅ Spot hung agents immediately
- ✅ See when agents finish
- ✅ Monitor multiple agents simultaneously

### 📊 Tracking Long-Running Jobs

```
┌─────────────────────────────────────────────────────────┐
│  ETL Pipeline ⠋    Database Sync    Report Gen ⠴       │
├─────────────────────────────────────────────────────────┤
│ ▶ Data Jobs ⠋    │  $ python etl_pipeline.py           │
│   Monitoring     │  Processing batch 3/10...            │
│   Backups        │  ⠋ 45% complete                      │
└─────────────────────────────────────────────────────────┘
```

**Benefits:**
- ✅ Track progress across multiple pipelines
- ✅ Catch stuck processes
- ✅ Know when jobs complete
- ✅ No need to switch tabs constantly

### 🔧 Development Workflows

```
┌─────────────────────────────────────────────────────────┐
│  npm run dev ⠋    cargo watch    pytest ⠴              │
├─────────────────────────────────────────────────────────┤
│ ▶ Dev Servers ⠋  │  $ npm run dev                      │
│   Tests          │  ⠋ Webpack compiling...              │
│   Logs           │  Server running on :3000             │
└─────────────────────────────────────────────────────────┘
```

**Benefits:**
- ✅ See which build is compiling
- ✅ Monitor test runs
- ✅ Track hot-reload cycles
- ✅ Debug parallel processes

## 🎨 Visual Design

### Spinner Characters

Using **Braille patterns** for smooth animation:

```
⠋  ⠙  ⠹  ⠸  ⠼  ⠴  ⠦  ⠧  ⠇  ⠏
```

**Why Braille?**
- ✅ Single character width (no layout shift)
- ✅ Visually distinct from text
- ✅ Smooth animation appearance
- ✅ Works in all terminal emulators

### Color Scheme

- **Active spinner**: Default foreground color (white/cyan)
- **Tab name**: Same as normal tabs
- **No color change**: Spinner itself is the indicator

**Design principle**: Subtle but noticeable. Doesn't distract, but catches the eye.

## 🔧 Technical Details

### Architecture

```
┌─────────────┐
│   Client    │
│  (main.rs)  │
└──────┬──────┘
       │ Every frame (80ms or 200ms)
       ↓
┌──────────────────────────────────┐
│  app.refresh_active_status()     │
│  → Query daemon via Unix socket  │
└──────┬───────────────────────────┘
       │
       ↓
┌─────────────────────────────────────┐
│  Daemon                             │
│  → Check last_output_time for each │
│  → Return tabs with output < 2s ago │
└─────────────────────────────────────┘
       │
       ↓
┌──────────────────────────────┐
│  UI Rendering                │
│  → Show spinner if active    │
│  → Update animation frame    │
└──────────────────────────────┘
```

### Code Flow

1. **Main loop** (`main.rs`):
   ```rust
   loop {
       app.refresh_active_status();  // Query daemon
       app.update_spinner();          // Advance animation
       terminal.draw(|f| draw(f, &mut app))?;
       
       let timeout = if app.has_active_tabs() {
           Duration::from_millis(80)   // Fast polling
       } else {
           Duration::from_millis(200)  // Slow polling
       };
       event::poll(timeout)?;
   }
   ```

2. **App state** (`app.rs`):
   ```rust
   pub struct App {
       pub active_tabs: HashSet<String>,  // Tab IDs with recent output
       pub spinner_frame: usize,           // 0-9
       pub last_spinner_update: Instant,
   }
   ```

3. **UI rendering** (`ui.rs`):
   ```rust
   let label = if app.active_tabs.contains(&tab.id) {
       format!("  {} {}  ", tab.name, app.get_spinner())
   } else {
       format!("  {}  ", tab.name)
   };
   ```

### Performance Optimization

**Adaptive polling** is key:

| Scenario | Behavior | Rationale |
|----------|----------|-----------|
| **All tabs idle** | Poll every 200ms (5 FPS) | Save CPU, no animation needed |
| **Any tab active** | Poll every 80ms (12.5 FPS) | Smooth spinner animation |

**Result**: 
- Smooth animation when needed
- Minimal CPU usage when idle
- No unnecessary daemon queries

## 🐛 Troubleshooting

### Spinner not showing

**Check:**
1. Is the command producing output?
   ```bash
   # This will show spinner
   ping google.com
   
   # This won't (no output)
   sleep 10
   ```

2. Is daemon running?
   ```bash
   mato --status
   ```

3. Check daemon logs:
   ```bash
   tail -f ~/.local/state/mato/daemon.log
   ```

### Spinner stuck/not animating

**Possible causes:**
- Client frozen (check CPU usage)
- Daemon not responding (restart daemon)
- Terminal emulator lag (try different terminal)

**Fix:**
```bash
# Restart daemon
pkill -f "mato.*daemon"
mato --daemon

# Restart client
mato
```

### Spinner disappears too quickly

**Current threshold**: 2 seconds

If you want longer persistence, modify `ACTIVE_THRESHOLD_SECS` in `src/client/app.rs`:

```rust
const ACTIVE_THRESHOLD_SECS: u64 = 5;  // Show spinner for 5 seconds
```

## 🔮 Future Enhancements

Potential improvements:

- [ ] **Configurable threshold** - User-defined active duration
- [ ] **Different spinners** - Per-task custom spinners
- [ ] **Color coding** - Different colors for different activity types
- [ ] **Sound alerts** - Beep when long task completes
- [ ] **Desktop notifications** - Notify when spinner disappears

## 📚 Related Documentation

- **[AI_AGENT_FRIENDLY.md](AI_AGENT_FRIENDLY.md)** - Why Mato is perfect for AI agents
- **[KEYBOARD_SHORTCUTS.md](KEYBOARD_SHORTCUTS.md)** - All keyboard shortcuts
- **[TERMINAL_PERSISTENCE.md](TERMINAL_PERSISTENCE.md)** - How persistence works

---

**TL;DR**: Spinners show which terminals are working. Perfect for monitoring AI agents and parallel tasks. Automatically adapts polling rate to save CPU.
