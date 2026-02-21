# 2026-02-22 Task → Desk Terminology Refactor

## 🎯 Overview

Complete terminology refactor: "Task" → "Desk" throughout the entire codebase. This change better reflects the concept of multiple workspaces/desks, each containing multiple terminal tabs.

## 🔄 Rationale

### Why "Desk" instead of "Task"?

**Conceptual Clarity**:
- "Task" implies a single job or action
- "Desk" implies a workspace with multiple tools (terminals)
- Better matches the mental model: multiple desks, each with multiple terminals

**User Mental Model**:
```
Before (confusing):
Task 1 → Terminal 1, Terminal 2, Terminal 3
Task 2 → Terminal 1, Terminal 2

After (clear):
Desk 1 → Terminal 1, Terminal 2, Terminal 3
Desk 2 → Terminal 1, Terminal 2
```

**Analogy**:
- Physical office: Multiple desks, each with multiple monitors
- Mato: Multiple desks, each with multiple terminals

## 📝 Changes

### 1. Core Data Structures

**Renamed Structs**:
```rust
// Before
pub struct Task {
    pub id: String,
    pub name: String,
    pub tabs: Vec<TabEntry>,
    pub active_tab: usize,
}

// After
pub struct Desk {
    pub id: String,
    pub name: String,
    pub tabs: Vec<TabEntry>,
    pub active_tab: usize,
}
```

**App State**:
```rust
// Before
pub struct App {
    pub tasks: Vec<Task>,
    pub new_task_area: Rect,
    // ...
}

// After
pub struct App {
    pub desks: Vec<Desk>,
    pub new_desk_area: Rect,
    // ...
}
```

### 2. Enums

**RenameTarget**:
```rust
// Before
pub enum RenameTarget { 
    Task(usize), 
    Tab(usize, usize) 
}

// After
pub enum RenameTarget { 
    Desk(usize), 
    Tab(usize, usize) 
}
```

### 3. Functions and Methods

| Before | After |
|--------|-------|
| `new_task()` | `new_desk()` |
| `close_task()` | `close_desk()` |
| `cur_task_mut()` | `cur_desk_mut()` |
| `active_task()` | `active_desk()` |
| `begin_rename_task()` | `begin_rename_desk()` |

### 4. Persistence Layer

**SavedState**:
```rust
// Before
#[derive(Serialize, Deserialize)]
pub struct SavedTask {
    pub id: String,
    pub name: String,
    pub tabs: Vec<SavedTab>,
    pub active_tab: usize,
}

pub struct SavedState {
    pub tasks: Vec<SavedTask>,
    pub active_task: usize,
}

// After
#[derive(Serialize, Deserialize)]
pub struct SavedDesk {
    pub id: String,
    pub name: String,
    pub tabs: Vec<SavedTab>,
    pub active_tab: usize,
}

pub struct SavedState {
    pub desks: Vec<SavedDesk>,
    pub active_desk: usize,
}
```

**state.json Format**:
```json
// Before
{
  "tasks": [
    {
      "id": "task-1",
      "name": "Task 1",
      "tabs": [...]
    }
  ],
  "active_task": 0
}

// After
{
  "desks": [
    {
      "id": "desk-1",
      "name": "Desk 1",
      "tabs": [...]
    }
  ],
  "active_desk": 0
}
```

### 5. UI Text Changes

| Before | After |
|--------|-------|
| "Tasks" (sidebar title) | "Desks" |
| "New Task" | "New Desk" |
| "Close Task" | "Close Desk" |
| "Rename Task" | "Rename Desk" |
| "Task 1", "Task 2" | "Desk 1", "Desk 2" |

**Sidebar**:
```
Before:                After:
┌─────────────┐       ┌─────────────┐
│ Tasks       │       │ Desks       │
│ ▶ Task 1    │       │ ▶ Desk 1    │
│   Task 2    │       │   Desk 2    │
└─────────────┘       └─────────────┘
```

**Status Bar**:
```
Before: [ n ] New Task  [ x ] Close  [ r ] Rename
After:  [ n ] New Desk  [ x ] Close  [ r ] Rename
```

## 🔧 Technical Details

### Files Modified

| File | Changes | Lines Changed |
|------|---------|---------------|
| `src/client/app.rs` | Struct rename, all methods | ~50 |
| `src/client/ui.rs` | UI text, rendering | ~20 |
| `src/client/input.rs` | Keyboard shortcuts | ~10 |
| `src/client/persistence.rs` | Serialization structs | ~15 |
| `src/main.rs` | Main loop, mouse handling | ~10 |
| **Total** | | **~105** |

### Migration Strategy

**Breaking Change**: Old `state.json` files are incompatible.

**Migration Steps**:
1. Delete old state file: `rm ~/.config/mato/state.json`
2. Restart mato
3. New state file will be created with "Desk" terminology

**No Data Loss**: PTY processes in daemon are unaffected. Only UI state is reset.

### Backward Compatibility

❌ **Not backward compatible** with old state files.

**Reason**: Field names changed (`tasks` → `desks`), causing deserialization errors.

**Impact**: Users will start with default "Desk 1" on first launch after update.

## 📊 Statistics

### Code Changes

```
Total files changed: 5
Total lines changed: ~105
  - Struct definitions: 2
  - Function names: 8
  - Variable names: ~30
  - UI strings: ~15
  - Comments: ~10
```

### Terminology Replacements

| Term | Occurrences | Replaced With |
|------|-------------|---------------|
| `Task` (struct) | 1 | `Desk` |
| `tasks` (field) | ~15 | `desks` |
| `new_task` | 3 | `new_desk` |
| `close_task` | 2 | `close_desk` |
| `"Task"` (UI) | ~8 | `"Desk"` |

## 🧪 Testing

### Manual Test Cases

1. **Fresh Start**
   - [ ] Delete `~/.config/mato/state.json`
   - [ ] Run mato
   - [ ] Verify sidebar shows "Desks" title
   - [ ] Verify default desk is "Desk 1"

2. **Create Desk**
   - [ ] Press `n` in sidebar
   - [ ] Verify new desk is "Desk 2"
   - [ ] Verify status bar shows "New Desk"

3. **Rename Desk**
   - [ ] Press `r` in sidebar
   - [ ] Verify popup shows "Rename Desk"
   - [ ] Type "Development"
   - [ ] Press Enter
   - [ ] Verify desk renamed

4. **Close Desk**
   - [ ] Create 2 desks
   - [ ] Press `x` to close one
   - [ ] Verify desk closed
   - [ ] Verify remaining desk still works

5. **Persistence**
   - [ ] Create 3 desks with custom names
   - [ ] Quit mato
   - [ ] Restart mato
   - [ ] Verify all desks restored
   - [ ] Check `~/.config/mato/state.json` contains `"desks"` field

### Automated Tests

No test changes required - all tests use internal APIs, not terminology.

## 🎨 User Impact

### Visual Changes

**Before**:
```
┌──────────────────────────────────────────────────────────┐
│ Tasks          │  Terminal 1    Terminal 2               │
│ ▶ Task 1       │                                          │
│   Task 2       │  $ npm run dev                           │
│   Task 3       │  > Starting server...                    │
└──────────────────────────────────────────────────────────┘
 [ n ] New Task  [ x ] Close  [ r ] Rename  [ q ] Quit
```

**After**:
```
┌──────────────────────────────────────────────────────────┐
│ Desks          │  Terminal 1    Terminal 2               │
│ ▶ Desk 1       │                                          │
│   Desk 2       │  $ npm run dev                           │
│   Desk 3       │  > Starting server...                    │
└──────────────────────────────────────────────────────────┘
 [ n ] New Desk  [ x ] Close  [ r ] Rename  [ q ] Quit
```

### Behavioral Changes

**None**. All functionality remains identical. Only terminology changed.

### Migration Experience

**First Launch After Update**:
1. Old state file ignored (incompatible format)
2. Starts with default "Desk 1"
3. User creates new desks as needed
4. New state file saved with "Desk" terminology

**User Action Required**: None. Automatic migration (reset to default).

## 💡 Design Rationale

### Why This Change?

**1. Conceptual Clarity**
- "Task" is ambiguous (a job? a workspace?)
- "Desk" is concrete (a workspace with tools)

**2. Better Mental Model**
- Physical analogy: office with multiple desks
- Each desk has multiple monitors (terminals)
- Natural grouping concept

**3. Consistency with Domain**
- Terminal multiplexers manage workspaces
- "Desk" better represents a workspace
- "Task" better represents a single job/command

**4. User Feedback**
- Users found "Task" confusing
- "What's the difference between a task and a tab?"
- "Desk" makes the hierarchy clearer

### Alternative Names Considered

| Name | Pros | Cons | Decision |
|------|------|------|----------|
| **Workspace** | Clear, common term | Too long, verbose | ❌ Rejected |
| **Project** | Familiar to developers | Implies code projects only | ❌ Rejected |
| **Session** | Used by tmux | Confusing with terminal sessions | ❌ Rejected |
| **Desk** | Short, concrete, clear | Less common | ✅ **Chosen** |
| **Group** | Generic | Too abstract | ❌ Rejected |

### Future Consistency

All future documentation, UI, and code will use "Desk" terminology consistently.

## 🔮 Future Work

### Documentation Updates

- [ ] Update README.md (Task → Desk)
- [ ] Update KEYBOARD_SHORTCUTS.md
- [ ] Update all release notes
- [ ] Update ACTIVITY_INDICATORS.md
- [ ] Update templates documentation

### Potential Enhancements

1. **Desk Icons** - Visual icons for different desk types
2. **Desk Templates** - Pre-configured desk layouts
3. **Desk Colors** - Color-code desks for quick identification
4. **Desk Shortcuts** - Quick switch between desks (Alt+1-9)

## 📚 Related Changes

This refactor is part of a larger effort to improve Mato's terminology and user experience:

- **v0.3.0**: Jump Mode, AI-agent-friendly design
- **v0.3.1**: Activity Indicators, PTY cleanup
- **v0.4.0** (this): Desk terminology refactor

## 🙏 Acknowledgments

**Inspiration**: Physical office workspace analogy - multiple desks, each with multiple monitors.

**User Feedback**: Community feedback that "Task" was confusing and didn't match the mental model.

---

**Version**: Unreleased (v0.4.0)  
**Date**: 2026-02-22  
**Type**: Breaking Change (state file format)  
**Migration**: Automatic (reset to default)  
**Impact**: UI terminology only, no functional changes
