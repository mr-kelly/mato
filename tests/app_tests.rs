/// Tests for Task and App business logic (tab/task management, rename, nav, idle).
/// Uses a NullProvider to avoid needing a live daemon socket.
use mato::terminal_provider::{ScreenContent, TerminalProvider};

struct NullProvider;
impl TerminalProvider for NullProvider {
    fn spawn(&mut self, _: u16, _: u16) {}
    fn resize(&mut self, _: u16, _: u16) {}
    fn write(&mut self, _: &[u8]) {}
    fn get_screen(&self, _: u16, _: u16) -> ScreenContent { ScreenContent::default() }
}

// ── helpers ──────────────────────────────────────────────────────────────────

use mato::client::app::{Focus, RenameTarget, TabEntry, Task};
use ratatui::widgets::ListState;
use std::collections::HashSet;

fn make_tab(name: &str) -> TabEntry {
    TabEntry { id: mato::utils::new_id(), name: name.into(), provider: Box::new(NullProvider) }
}

fn make_task(name: &str) -> Task {
    Task { id: mato::utils::new_id(), name: name.into(), tabs: vec![make_tab("Terminal 1")], active_tab: 0 }
}

// ── Task: tab management ─────────────────────────────────────────────────────

#[test]
fn close_tab_cannot_remove_last() {
    let mut task = make_task("T");
    assert_eq!(task.tabs.len(), 1);
    task.close_tab();
    assert_eq!(task.tabs.len(), 1, "should not remove the last tab");
}

#[test]
fn close_tab_adjusts_active_index() {
    let mut task = make_task("T");
    task.tabs.push(make_tab("Tab 2"));
    task.tabs.push(make_tab("Tab 3"));
    task.active_tab = 2; // select last tab
    task.close_tab();
    assert_eq!(task.tabs.len(), 2);
    assert_eq!(task.active_tab, 1, "active_tab should clamp after removal");
}

#[test]
fn new_tab_selects_new_tab() {
    let mut task = make_task("T");
    task.new_tab();
    assert_eq!(task.tabs.len(), 2);
    assert_eq!(task.active_tab, 1, "new tab should become active");
}

// ── App: task management ─────────────────────────────────────────────────────

fn make_app_with(tasks: Vec<Task>) -> mato::client::app::App {
    let mut app = mato::client::app::App::new();
    app.tasks = tasks;
    app.list_state.select(Some(0));
    app
}

#[test]
fn close_task_cannot_remove_last() {
    let mut app = make_app_with(vec![make_task("Only")]);
    app.close_task();
    assert_eq!(app.tasks.len(), 1);
}

#[test]
fn close_task_selects_valid_index_after_removal() {
    let mut app = make_app_with(vec![make_task("A"), make_task("B"), make_task("C")]);
    // select last task then close it
    app.nav(2);
    assert_eq!(app.selected(), 2);
    app.close_task();
    assert_eq!(app.tasks.len(), 2);
    assert_eq!(app.selected(), 1, "selection should clamp to new last");
}

#[test]
fn nav_does_not_go_out_of_bounds() {
    let mut app = make_app_with(vec![make_task("A"), make_task("B")]);
    app.nav(-99);
    assert_eq!(app.selected(), 0);
    app.nav(99);
    assert_eq!(app.selected(), 1);
}

// ── App: rename ───────────────────────────────────────────────────────────────

#[test]
fn commit_rename_empty_string_is_ignored() {
    let mut app = make_app_with(vec![make_task("Original")]);
    app.rename = Some((RenameTarget::Task(0), "   ".into()));
    app.commit_rename();
    assert_eq!(app.tasks[0].name, "Original", "empty rename should not apply");
    assert!(!app.dirty);
}

#[test]
fn commit_rename_task_applies_trimmed_name() {
    let mut app = make_app_with(vec![make_task("Old")]);
    app.rename = Some((RenameTarget::Task(0), "  New Name  ".into()));
    app.commit_rename();
    assert_eq!(app.tasks[0].name, "New Name");
    assert!(app.dirty);
}

#[test]
fn cancel_rename_clears_state() {
    let mut app = make_app_with(vec![make_task("T")]);
    app.rename = Some((RenameTarget::Task(0), "typing...".into()));
    app.cancel_rename();
    assert!(app.rename.is_none());
}

// ── Idle detection: threshold filtering ──────────────────────────────────────

#[test]
fn idle_tabs_only_marks_above_threshold() {
    // Simulate what refresh_idle_status does: filter tabs >= threshold
    const THRESHOLD: u64 = 30;
    let raw: Vec<(String, u64)> = vec![
        ("tab-a".into(), 5),   // active
        ("tab-b".into(), 30),  // exactly at threshold → idle
        ("tab-c".into(), 60),  // idle
    ];
    let idle: HashSet<String> = raw.into_iter()
        .filter(|(_, secs)| *secs >= THRESHOLD)
        .map(|(id, _)| id)
        .collect();

    assert!(!idle.contains("tab-a"));
    assert!(idle.contains("tab-b"));
    assert!(idle.contains("tab-c"));
}

#[test]
fn task_is_idle_only_when_all_tabs_idle() {
    let mut task = make_task("T");
    task.tabs.push(make_tab("Tab 2"));

    let mut idle_tabs = HashSet::new();
    idle_tabs.insert(task.tabs[0].id.clone());
    // only first tab idle → task not fully idle
    let all_idle = task.tabs.iter().all(|t| idle_tabs.contains(&t.id));
    assert!(!all_idle);

    idle_tabs.insert(task.tabs[1].id.clone());
    let all_idle = task.tabs.iter().all(|t| idle_tabs.contains(&t.id));
    assert!(all_idle);
}

// ── Vt100Emulator: basic output rendering ────────────────────────────────────

#[test]
fn vt100_renders_written_text() {
    use mato::terminal_emulator::TerminalEmulator;
    use mato::emulators::Vt100Emulator;

    let mut emu = Vt100Emulator::new(24, 80);
    emu.process(b"Hello");
    let screen = emu.get_screen(24, 80);

    let first_row: String = screen.lines[0].cells.iter().map(|c| c.ch).collect();
    assert!(first_row.starts_with("Hello"), "first row should start with 'Hello', got: {first_row:?}");
}

#[test]
fn vt100_cursor_advances_after_text() {
    use mato::terminal_emulator::TerminalEmulator;
    use mato::emulators::Vt100Emulator;

    let mut emu = Vt100Emulator::new(24, 80);
    emu.process(b"Hi");
    let screen = emu.get_screen(24, 80);
    let (_, col) = screen.cursor;
    assert_eq!(col, 2, "cursor column should be 2 after writing 2 chars");
}

// ── Persistence: SavedState round-trip ───────────────────────────────────────

#[test]
fn saved_state_roundtrip() {
    use mato::client::persistence::{SavedState, SavedTab, SavedTask};

    let state = SavedState {
        active_task: 1,
        tasks: vec![
            SavedTask {
                id: "t1".into(),
                name: "Work".into(),
                active_tab: 0,
                tabs: vec![SavedTab { id: "tb1".into(), name: "Terminal 1".into() }],
            },
            SavedTask {
                id: "t2".into(),
                name: "Personal".into(),
                active_tab: 1,
                tabs: vec![
                    SavedTab { id: "tb2".into(), name: "Terminal 1".into() },
                    SavedTab { id: "tb3".into(), name: "Terminal 2".into() },
                ],
            },
        ],
    };

    let json = serde_json::to_string(&state).unwrap();
    let restored: SavedState = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.active_task, 1);
    assert_eq!(restored.tasks.len(), 2);
    assert_eq!(restored.tasks[1].name, "Personal");
    assert_eq!(restored.tasks[1].tabs.len(), 2);
    assert_eq!(restored.tasks[1].active_tab, 1);
}
