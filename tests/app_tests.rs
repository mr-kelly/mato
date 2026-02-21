/// Tests for Desk and App business logic (tab/task management, rename, nav, idle).
/// Uses a NullProvider to avoid needing a live daemon socket.
use mato::terminal_provider::{ScreenContent, TerminalProvider};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct NullProvider;
impl TerminalProvider for NullProvider {
    fn spawn(&mut self, _: u16, _: u16) {}
    fn resize(&mut self, _: u16, _: u16) {}
    fn write(&mut self, _: &[u8]) {}
    fn get_screen(&self, _: u16, _: u16) -> ScreenContent { ScreenContent::default() }
}

struct CountingMouseProvider {
    calls: Arc<AtomicUsize>,
    enabled: bool,
}
impl TerminalProvider for CountingMouseProvider {
    fn spawn(&mut self, _: u16, _: u16) {}
    fn resize(&mut self, _: u16, _: u16) {}
    fn write(&mut self, _: &[u8]) {}
    fn mouse_mode_enabled(&self) -> bool {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.enabled
    }
    fn get_screen(&self, _: u16, _: u16) -> ScreenContent { ScreenContent::default() }
}

// ── helpers ──────────────────────────────────────────────────────────────────

use mato::client::app::{RenameTarget, TabEntry, Desk};
use std::collections::HashSet;

fn make_tab(name: &str) -> TabEntry {
    TabEntry { id: mato::utils::new_id(), name: name.into(), provider: Box::new(NullProvider) }
}

fn make_task(name: &str) -> Desk {
    Desk { id: mato::utils::new_id(), name: name.into(), tabs: vec![make_tab("Terminal 1")], active_tab: 0 }
}

fn make_mouse_mode_app(calls: Arc<AtomicUsize>) -> mato::client::app::App {
    let tab = TabEntry {
        id: mato::utils::new_id(),
        name: "Terminal 1".into(),
        provider: Box::new(CountingMouseProvider { calls, enabled: true }),
    };
    let desk = Desk {
        id: mato::utils::new_id(),
        name: "Desk".into(),
        tabs: vec![tab],
        active_tab: 0,
    };
    make_app_with(vec![desk])
}

// ── Desk: tab management ─────────────────────────────────────────────────────

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

fn make_app_with(desks: Vec<Desk>) -> mato::client::app::App {
    let mut app = mato::client::app::App::new();
    app.current_office = 0;
    // Ensure exactly one office with the given desks
    app.offices = vec![mato::client::app::Office {
        id: "test".into(), name: "Test".into(), desks, active_desk: 0,
    }];
    app.list_state.select(Some(0));
    app
}

#[test]
fn close_task_cannot_remove_last() {
    let mut app = make_app_with(vec![make_task("Only")]);
    app.close_desk();
    assert_eq!(app.offices[0].desks.len(), 1);
}

#[test]
fn close_task_selects_valid_index_after_removal() {
    let mut app = make_app_with(vec![make_task("A"), make_task("B"), make_task("C")]);
    // select last task then close it
    app.nav(2);
    assert_eq!(app.selected(), 2);
    app.close_desk();
    assert_eq!(app.offices[0].desks.len(), 2);
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
    app.rename = Some((RenameTarget::Desk(0), "   ".into()));
    app.commit_rename();
    assert_eq!(app.offices[0].desks[0].name, "Original", "empty rename should not apply");
    assert!(!app.dirty);
}

#[test]
fn commit_rename_task_applies_trimmed_name() {
    let mut app = make_app_with(vec![make_task("Old")]);
    app.rename = Some((RenameTarget::Desk(0), "  New Name  ".into()));
    app.commit_rename();
    assert_eq!(app.offices[0].desks[0].name, "New Name");
    assert!(app.dirty);
}

#[test]
fn cancel_rename_clears_state() {
    let mut app = make_app_with(vec![make_task("T")]);
    app.rename = Some((RenameTarget::Desk(0), "typing...".into()));
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
    use mato::client::persistence::{SavedState, SavedTab, SavedDesk, SavedOffice};

    let state = SavedState {
        current_office: 0,
        offices: vec![SavedOffice {
            id: "o1".into(), name: "Default".into(), active_desk: 1,
            desks: vec![
                SavedDesk {
                    id: "t1".into(), name: "Work".into(), active_tab: 0,
                    tabs: vec![SavedTab { id: "tb1".into(), name: "Terminal 1".into() }],
                },
                SavedDesk {
                    id: "t2".into(), name: "Personal".into(), active_tab: 1,
                    tabs: vec![
                        SavedTab { id: "tb2".into(), name: "Terminal 1".into() },
                        SavedTab { id: "tb3".into(), name: "Terminal 2".into() },
                    ],
                },
            ],
        }],
    };

    let json = serde_json::to_string(&state).unwrap();
    let restored: SavedState = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.offices[0].active_desk, 1);
    assert_eq!(restored.offices[0].desks.len(), 2);
    assert_eq!(restored.offices[0].desks[1].name, "Personal");
    assert_eq!(restored.offices[0].desks[1].tabs.len(), 2);
    assert_eq!(restored.offices[0].desks[1].active_tab, 1);
}

#[test]
fn mouse_mode_query_is_cached_briefly() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut app = make_mouse_mode_app(calls.clone());

    assert!(app.pty_mouse_mode_enabled());
    assert!(app.pty_mouse_mode_enabled());

    assert_eq!(calls.load(Ordering::Relaxed), 1, "second call should hit cache");
}
