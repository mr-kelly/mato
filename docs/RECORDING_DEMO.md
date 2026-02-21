# Recording Demo GIF for README

## 🎯 Goal

Create a compelling demo GIF showing **Activity Indicators** in action - the key differentiator of Mato.

## 📹 What to Show

### Scene 1: Multiple AI Agents (10 seconds)

**Setup:**
```bash
# Task 1: Claude Agent
$ claude "Analyze this codebase"

# Task 2: Codex CLI  
$ codex "Generate API tests"

# Task 3: GitHub Copilot
$ gh copilot suggest "optimize database queries"
```

**What viewers see:**
```
┌─────────────────────────────────────────────────────────┐
│  Claude Agent ⠋    Codex CLI ⠹    Copilot ⠴           │
├─────────────────────────────────────────────────────────┤
│ ▶ AI Agents ⠋    │  $ claude "Analyze this codebase"   │
│   Development    │  ⠋ Reading files...                  │
│   Testing        │  ⠋ Analyzing patterns...             │
└─────────────────────────────────────────────────────────┘
```

**Key points:**
- ✅ Show spinners animating on active tabs
- ✅ Show sidebar task also has spinner
- ✅ Show one agent finishing (spinner disappears)

### Scene 2: Long-Running Tasks (8 seconds)

**Setup:**
```bash
# Task 1: Build
$ npm run build

# Task 2: Tests
$ cargo test --all

# Task 3: Database
$ pg_dump large_db > backup.sql
```

**What viewers see:**
- Spinners on all 3 tabs initially
- Build finishes → spinner disappears
- Tests still running → spinner continues
- Database backup → spinner continues

### Scene 3: Jump Mode Navigation (5 seconds)

**Show:**
1. Press `Esc` → Jump Mode activates
2. Labels appear: `[a] [b] [c] [d]`
3. Press `c` → Jump to that tab instantly
4. Show spinner still animating on active tab

## 🛠️ Recording Tools

### Option 1: asciinema + agg (Recommended)

```bash
# Install
cargo install agg

# Record
asciinema rec demo.cast

# Convert to GIF
agg demo.cast demo.gif --speed 1.5 --font-size 16
```

**Pros:**
- ✅ Perfect terminal rendering
- ✅ Small file size
- ✅ Easy to edit timing

### Option 2: terminalizer

```bash
# Install
npm install -g terminalizer

# Record
terminalizer record demo

# Render
terminalizer render demo -o demo.gif
```

**Pros:**
- ✅ Beautiful themes
- ✅ Easy to configure

### Option 3: peek (Linux GUI)

```bash
# Install
sudo apt install peek

# Record
# Just click and record the terminal window
```

**Pros:**
- ✅ Simple GUI
- ✅ Real-time preview

## 📐 Specifications

| Setting | Value | Reason |
|---------|-------|--------|
| **Resolution** | 800x500 px | Fits README width |
| **FPS** | 15-20 | Smooth spinner animation |
| **Duration** | 20-25 seconds | Short attention span |
| **Font Size** | 14-16 pt | Readable on GitHub |
| **Theme** | Mato default (navy) | Brand consistency |
| **Speed** | 1.5x | Keep it snappy |

## 🎬 Recording Script

### Preparation

```bash
# 1. Clean terminal
clear

# 2. Set terminal size
resize -s 30 100

# 3. Start Mato with fresh state
rm -rf ~/.config/mato/state.json
mato

# 4. Create 3 tasks
# Task 1: AI Agents
# Task 2: Development  
# Task 3: Data Processing
```

### Recording Steps

**Step 1: Show idle state (2s)**
- All tabs visible, no spinners
- Clean interface

**Step 2: Start multiple tasks (5s)**
- Switch to tab 1: `Esc → a`
- Run: `ping google.com` (continuous output)
- Switch to tab 2: `Esc → b`
- Run: `for i in {1..100}; do echo "Processing $i"; sleep 0.1; done`
- Switch to tab 3: `Esc → c`
- Run: `watch -n 1 date`

**Step 3: Show spinners (8s)**
- Navigate back to sidebar: `Esc → ←`
- Show all 3 tasks with spinners
- Hover over tabs to show spinners in topbar
- Let spinners animate for a few seconds

**Step 4: Show completion (3s)**
- Stop one task: `Esc → a → Ctrl+C`
- Show spinner disappears after 2 seconds
- Other spinners still animating

**Step 5: Jump Mode demo (5s)**
- Press `Esc` → Show labels
- Press `d` → Jump to tab
- Show spinner still there

**Step 6: End (2s)**
- Show final state with mixed active/idle tabs

## 🎨 Post-Processing

### Optimize GIF Size

```bash
# Using gifsicle
gifsicle -O3 --colors 256 demo.gif -o demo-optimized.gif

# Using ImageMagick
convert demo.gif -fuzz 10% -layers Optimize demo-optimized.gif
```

**Target size:** < 5 MB (GitHub limit: 10 MB)

### Add Annotations (Optional)

Use tools like:
- **Gifski** - High quality GIF encoder
- **LICEcap** - Add text overlays
- **ScreenToGif** - Frame-by-frame editing

## ✅ Quality Checklist

Before publishing:

- [ ] Spinners are clearly visible and animating smoothly
- [ ] At least 3 different tabs shown with spinners
- [ ] Sidebar task spinner visible
- [ ] One spinner disappears (shows completion)
- [ ] Jump Mode labels clearly visible
- [ ] File size < 5 MB
- [ ] Duration 20-25 seconds
- [ ] No sensitive information visible
- [ ] Terminal size consistent (800x500)
- [ ] Colors match Mato theme

## 📝 Alternative: Static Screenshot

If GIF is too complex, use a static screenshot with annotations:

```
┌─────────────────────────────────────────────────────────┐
│  Agent 1 ⠋    Agent 2    Agent 3 ⠴    Agent 4          │
│           ↑                      ↑                       │
│      Working!              Working!                      │
├─────────────────────────────────────────────────────────┤
│ ▶ Development ⠋  │                                      │
│   Testing        │  $ npm run dev                       │
│   Production ⠴   │  ⠋ Compiling...                      │
│        ↑         │                                      │
│   Has active     │                                      │
│   terminals!     │                                      │
└─────────────────────────────────────────────────────────┘
```

**Tools:**
- `scrot` - Screenshot
- GIMP - Add arrows and text
- Inkscape - Vector annotations

## 🚀 Publishing

```bash
# Save to docs/
cp demo-optimized.gif docs/demo.gif

# Update README.md
# (Already done - just replace placeholder)

# Commit
git add docs/demo.gif README.md
git commit -m "docs: Add activity indicators demo GIF"
git push
```

## 📚 References

- [asciinema](https://asciinema.org/)
- [agg](https://github.com/asciinema/agg)
- [terminalizer](https://terminalizer.com/)
- [peek](https://github.com/phw/peek)
- [gifsicle](https://www.lcdf.org/gifsicle/)

---

**Pro Tip:** Record multiple takes and pick the best one. The first take is rarely perfect!
