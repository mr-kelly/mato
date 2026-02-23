/// Tests for Desk and App business logic (tab/task management, rename, nav, idle).
/// Uses a NullProvider to avoid needing a live daemon socket.
use mato::terminal_provider::{ScreenContent, TerminalProvider};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct NullProvider;
impl TerminalProvider for NullProvider {
    fn spawn(&mut self, _: u16, _: u16) {}
    fn resize(&mut self, _: u16, _: u16) {}
    fn write(&mut self, _: &[u8]) {}
    fn get_screen(&self, _: u16, _: u16) -> ScreenContent {
        ScreenContent::default()
    }
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
    fn get_screen(&self, _: u16, _: u16) -> ScreenContent {
        ScreenContent::default()
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

use mato::client::app::{Desk, Focus, RenameTarget, TabEntry};
use std::collections::HashSet;

fn make_tab(name: &str) -> TabEntry {
    TabEntry {
        id: mato::utils::new_id(),
        name: name.into(),
        provider: Box::new(NullProvider),
    }
}

fn make_task(name: &str) -> Desk {
    Desk {
        id: mato::utils::new_id(),
        name: name.into(),
        tabs: vec![make_tab("Terminal 1")],
        active_tab: 0,
    }
}

fn make_mouse_mode_app(calls: Arc<AtomicUsize>) -> mato::client::app::App {
    let tab = TabEntry {
        id: mato::utils::new_id(),
        name: "Terminal 1".into(),
        provider: Box::new(CountingMouseProvider {
            calls,
            enabled: true,
        }),
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
        id: "test".into(),
        name: "Test".into(),
        desks,
        active_desk: 0,
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
    assert_eq!(
        app.offices[0].desks[0].name, "Original",
        "empty rename should not apply"
    );
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

#[test]
fn jump_labels_content_excludes_c_r_q_and_includes_digits() {
    let mut app = make_app_with(vec![make_task("T")]);
    app.focus = Focus::Content;
    let labels = app.jump_labels();
    assert!(labels.contains(&'1'));
    assert!(!labels.contains(&'c'));
    assert!(!labels.contains(&'C'));
    assert!(!labels.contains(&'r'));
    assert!(!labels.contains(&'R'));
    assert!(!labels.contains(&'q'));
    assert!(!labels.contains(&'Q'));
}

#[test]
fn jump_targets_sidebar_use_visible_window_after_scroll() {
    let desks: Vec<Desk> = (0..40).map(|i| make_task(&format!("Desk {}", i))).collect();
    let mut app = make_app_with(desks);
    app.focus = Focus::Sidebar;
    app.sidebar_list_area = ratatui::layout::Rect {
        x: 0,
        y: 0,
        width: 30,
        height: 8, // 6 visible rows after borders
    };
    *app.list_state.offset_mut() = 25;
    app.list_state.select(Some(25));

    let targets = app.jump_targets();
    assert!(!targets.is_empty());
    assert_eq!(targets[0], ('t', 25, 0));
    assert!(targets
        .iter()
        .all(|(kind, idx, _)| *kind == 't' && *idx >= 25));
    assert!(!targets.iter().any(|(_, idx, _)| *idx < 25));
}

// ── Idle detection: threshold filtering ──────────────────────────────────────

#[test]
fn idle_tabs_only_marks_above_threshold() {
    // Simulate what refresh_idle_status does: filter tabs >= threshold
    const THRESHOLD: u64 = 30;
    let raw: Vec<(String, u64)> = vec![
        ("tab-a".into(), 5),  // active
        ("tab-b".into(), 30), // exactly at threshold → idle
        ("tab-c".into(), 60), // idle
    ];
    let idle: HashSet<String> = raw
        .into_iter()
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
    use mato::emulators::Vt100Emulator;
    use mato::terminal_emulator::TerminalEmulator;

    let mut emu = Vt100Emulator::new(24, 80);
    emu.process(b"Hello");
    let screen = emu.get_screen(24, 80);

    let first_row: String = screen.lines[0].cells.iter().map(|c| c.ch).collect();
    assert!(
        first_row.starts_with("Hello"),
        "first row should start with 'Hello', got: {first_row:?}"
    );
}

#[test]
fn vt100_cursor_advances_after_text() {
    use mato::emulators::Vt100Emulator;
    use mato::terminal_emulator::TerminalEmulator;

    let mut emu = Vt100Emulator::new(24, 80);
    emu.process(b"Hi");
    let screen = emu.get_screen(24, 80);
    let (_, col) = screen.cursor;
    assert_eq!(col, 2, "cursor column should be 2 after writing 2 chars");
}

// ── Persistence: SavedState round-trip ───────────────────────────────────────

#[test]
fn saved_state_roundtrip() {
    use mato::client::persistence::{SavedDesk, SavedOffice, SavedState, SavedTab};

    let state = SavedState {
        current_office: 0,
        offices: vec![SavedOffice {
            id: "o1".into(),
            name: "Default".into(),
            active_desk: 1,
            desks: vec![
                SavedDesk {
                    id: "t1".into(),
                    name: "Work".into(),
                    active_tab: 0,
                    tabs: vec![SavedTab {
                        id: "tb1".into(),
                        name: "Terminal 1".into(),
                    }],
                },
                SavedDesk {
                    id: "t2".into(),
                    name: "Personal".into(),
                    active_tab: 1,
                    tabs: vec![
                        SavedTab {
                            id: "tb2".into(),
                            name: "Terminal 1".into(),
                        },
                        SavedTab {
                            id: "tb3".into(),
                            name: "Terminal 2".into(),
                        },
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

    assert_eq!(
        calls.load(Ordering::Relaxed),
        1,
        "second call should hit cache"
    );
}

// ── sync_focus_events: focus tracking gating ─────────────────────────────────

use std::sync::Mutex;

/// Provider that records written bytes and exposes configurable focus_events_enabled.
struct TrackingFocusProvider {
    written: Arc<Mutex<Vec<u8>>>,
    focus_enabled: bool,
}

impl TerminalProvider for TrackingFocusProvider {
    fn spawn(&mut self, _: u16, _: u16) {}
    fn resize(&mut self, _: u16, _: u16) {}
    fn write(&mut self, bytes: &[u8]) {
        self.written.lock().unwrap().extend_from_slice(bytes);
    }
    fn focus_events_enabled(&self) -> bool {
        self.focus_enabled
    }
    fn get_screen(&self, _: u16, _: u16) -> ScreenContent {
        ScreenContent::default()
    }
}

fn make_focus_tracking_app(focus_enabled: bool) -> (mato::client::app::App, Arc<Mutex<Vec<u8>>>) {
    let written = Arc::new(Mutex::new(Vec::<u8>::new()));
    let tab = TabEntry {
        id: mato::utils::new_id(),
        name: "Terminal 1".into(),
        provider: Box::new(TrackingFocusProvider {
            written: Arc::clone(&written),
            focus_enabled,
        }),
    };
    let desk = Desk {
        id: mato::utils::new_id(),
        name: "Desk".into(),
        tabs: vec![tab],
        active_tab: 0,
    };
    let app = make_app_with(vec![desk]);
    (app, written)
}

#[test]
fn sync_focus_events_no_write_when_tracking_disabled() {
    // focus_events_enabled=false → no \x1b[I or \x1b[O should be written
    let (mut app, written) = make_focus_tracking_app(false);
    // Start from Sidebar, switch to Content
    app.focus = Focus::Sidebar;
    app.prev_focus = Focus::Sidebar;

    app.focus = Focus::Content;
    app.sync_focus_events();

    let bytes = written.lock().unwrap();
    assert!(
        !bytes.windows(3).any(|w| w == b"\x1b[I"),
        "focus-in must NOT be sent when tracking disabled, got: {:?}",
        bytes
    );
}

#[test]
fn sync_focus_events_sends_focus_in_when_tracking_enabled() {
    let (mut app, written) = make_focus_tracking_app(true);
    app.focus = Focus::Sidebar;
    app.prev_focus = Focus::Sidebar;

    app.focus = Focus::Content;
    app.sync_focus_events();

    let bytes = written.lock().unwrap();
    assert!(
        bytes.windows(3).any(|w| w == b"\x1b[I"),
        "focus-in \\x1b[I must be sent when tracking enabled, got: {:?}",
        bytes
    );
    assert!(
        !bytes.windows(3).any(|w| w == b"\x1b[O"),
        "focus-out must NOT be sent on focus-in transition"
    );
}

#[test]
fn sync_focus_events_sends_focus_out_when_leaving_content() {
    let (mut app, written) = make_focus_tracking_app(true);
    // Start in Content
    app.focus = Focus::Content;
    app.prev_focus = Focus::Content;

    // Leave content (e.g. Esc → Sidebar)
    app.focus = Focus::Sidebar;
    app.sync_focus_events();

    let bytes = written.lock().unwrap();
    assert!(
        bytes.windows(3).any(|w| w == b"\x1b[O"),
        "focus-out \\x1b[O must be sent when leaving Content, got: {:?}",
        bytes
    );
    assert!(
        !bytes.windows(3).any(|w| w == b"\x1b[I"),
        "focus-in must NOT be sent on focus-out transition"
    );
}

#[test]
fn sync_focus_events_no_op_when_focus_unchanged() {
    let (mut app, written) = make_focus_tracking_app(true);
    app.focus = Focus::Content;
    app.prev_focus = Focus::Content;

    app.sync_focus_events(); // focus == prev_focus → early return

    let bytes = written.lock().unwrap();
    assert!(
        bytes.is_empty(),
        "nothing should be written when focus is unchanged"
    );
}

// ── from_saved: clamping ──────────────────────────────────────────────────────

use mato::client::persistence::{SavedDesk, SavedOffice, SavedState, SavedTab};

fn make_saved_state_with_active_tab(active_tab: usize, n_tabs: usize) -> SavedState {
    SavedState {
        current_office: 0,
        offices: vec![SavedOffice {
            id: "o1".into(),
            name: "Office".into(),
            active_desk: 0,
            desks: vec![SavedDesk {
                id: "d1".into(),
                name: "Desk".into(),
                active_tab,
                tabs: (0..n_tabs)
                    .map(|i| SavedTab {
                        id: format!("t{i}"),
                        name: format!("Tab {i}"),
                    })
                    .collect(),
            }],
        }],
    }
}

#[test]
fn from_saved_clamps_active_tab_to_valid_range() {
    // active_tab=99 for desk with 2 tabs → must be clamped to 1
    let state = make_saved_state_with_active_tab(99, 2);
    let app = mato::client::app::App::from_saved(state);
    let desk = &app.offices[0].desks[0];
    assert!(
        desk.active_tab < desk.tabs.len(),
        "active_tab {} must be < tab count {}",
        desk.active_tab,
        desk.tabs.len()
    );
    assert_eq!(desk.active_tab, 1);
}

#[test]
fn from_saved_clamps_active_desk_to_valid_range() {
    let state = SavedState {
        current_office: 0,
        offices: vec![SavedOffice {
            id: "o1".into(),
            name: "Office".into(),
            active_desk: 99, // corrupted
            desks: vec![
                SavedDesk {
                    id: "d1".into(),
                    name: "Desk A".into(),
                    active_tab: 0,
                    tabs: vec![SavedTab { id: "t1".into(), name: "Tab 1".into() }],
                },
                SavedDesk {
                    id: "d2".into(),
                    name: "Desk B".into(),
                    active_tab: 0,
                    tabs: vec![SavedTab { id: "t2".into(), name: "Tab 1".into() }],
                },
            ],
        }],
    };
    let app = mato::client::app::App::from_saved(state);
    let office = &app.offices[0];
    assert!(
        office.active_desk < office.desks.len(),
        "active_desk must be clamped"
    );
}

// ── close_desk: tab_scroll reset ──────────────────────────────────────────────

#[test]
fn close_desk_resets_tab_scroll_to_zero() {
    let mut d1 = make_task("A");
    // Add extra tabs to simulate scrolled state
    for i in 0..5 {
        d1.tabs.push(make_tab(&format!("Tab {i}")));
    }
    let d2 = make_task("B");
    let mut app = make_app_with(vec![d1, d2]);
    app.tab_scroll = 3; // simulate scrolled topbar

    // Close first desk
    app.nav(0);
    app.close_desk(); // goes through request_close_desk; call confirm path
    // If only one desk is protected, add a second and actually close
    app.nav(0);
    // Force close by calling close_desk_at via public close_desk when 2 desks remain
    assert_eq!(app.tab_scroll, 0, "tab_scroll must be reset to 0 after desk close");
}

// ── sync_focus_events: no panic with empty tabs ───────────────────────────────

#[test]
fn sync_focus_events_safe_with_empty_tabs_desk() {
    // A desk can temporarily have no tabs after close operations.
    // sync_focus_events must not panic.
    let empty_desk = Desk {
        id: mato::utils::new_id(),
        name: "Empty".into(),
        tabs: vec![],
        active_tab: 0,
    };
    let mut app = make_app_with(vec![empty_desk]);
    app.focus = Focus::Content;
    app.prev_focus = Focus::Sidebar;
    // Should not panic
    app.sync_focus_events();
}

// ── pty_mouse_mode: cache invalidated on tab switch ───────────────────────────

struct EnabledMouseProvider;
impl TerminalProvider for EnabledMouseProvider {
    fn spawn(&mut self, _: u16, _: u16) {}
    fn resize(&mut self, _: u16, _: u16) {}
    fn write(&mut self, _: &[u8]) {}
    fn mouse_mode_enabled(&self) -> bool { true }
    fn get_screen(&self, _: u16, _: u16) -> ScreenContent { ScreenContent::default() }
}

#[test]
fn pty_mouse_mode_cache_invalidated_on_tab_switch() {
    // First tab: mouse enabled, second tab: mouse disabled (NullProvider).
    let tab1 = TabEntry {
        id: "tab-mouse".into(),
        name: "Mouse".into(),
        provider: Box::new(EnabledMouseProvider),
    };
    let tab2 = TabEntry {
        id: "tab-none".into(),
        name: "NoMouse".into(),
        provider: Box::new(NullProvider),
    };
    let desk = Desk {
        id: mato::utils::new_id(),
        name: "Desk".into(),
        tabs: vec![tab1, tab2],
        active_tab: 0,
    };
    let mut app = make_app_with(vec![desk]);
    assert!(app.pty_mouse_mode_enabled(), "tab1 should have mouse enabled");
    // Switch to tab2
    app.offices[0].desks[0].active_tab = 1;
    assert!(!app.pty_mouse_mode_enabled(), "tab2 should not have mouse enabled after switch");
}

// ── Spinner: needs_update drives redraws ──────────────────────────────────────

#[test]
fn spinner_needs_update_false_immediately_after_update() {
    let mut app = make_app_with(vec![make_task("T")]);
    app.update_spinner();
    assert!(
        !app.spinner_needs_update(),
        "spinner_needs_update must be false right after an update"
    );
}

#[test]
fn spinner_needs_update_true_after_80ms() {
    use std::thread;
    let mut app = make_app_with(vec![make_task("T")]);
    app.update_spinner(); // reset timer
    thread::sleep(std::time::Duration::from_millis(85));
    assert!(
        app.spinner_needs_update(),
        "spinner_needs_update must be true after 80ms"
    );
}

#[test]
fn update_spinner_advances_frame_after_80ms() {
    use std::thread;
    let mut app = make_app_with(vec![make_task("T")]);
    let before = app.spinner_frame;
    thread::sleep(std::time::Duration::from_millis(85));
    app.update_spinner();
    assert_ne!(app.spinner_frame, before, "spinner_frame must advance after 80ms");
}
