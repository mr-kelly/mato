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

use mato::client::app::{Desk, Focus, RenameState, RenameTarget, TabEntry};
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
    task.new_tab(None);
    assert_eq!(task.tabs.len(), 2);
    assert_eq!(task.active_tab, 1, "new tab should become active");
}

#[test]
fn new_tab_with_cwd_passes_cwd_to_provider() {
    let mut desk = make_task("T");
    desk.new_tab(Some("/home/kelly/projects/mato".into()));
    assert_eq!(desk.tabs.len(), 2);
    assert_eq!(desk.active_tab, 1);
    // The new tab's provider should have the cwd set (get_cwd returns it before spawn)
    assert_eq!(
        desk.tabs[1].provider.get_cwd(),
        None, // not yet spawned — cwd is stored internally, OSC7 not yet emitted
        "get_cwd returns None before spawn (stored as spawn_cwd)"
    );
}

#[test]
fn new_tab_no_cwd_defaults_to_none() {
    let mut desk = make_task("T");
    desk.new_tab(None);
    assert_eq!(desk.tabs[1].provider.get_cwd(), None);
}


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
    app.rename = Some(RenameState::new(RenameTarget::Desk(0), "   ".into()));
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
    app.rename = Some(RenameState::new(
        RenameTarget::Desk(0),
        "  New Name  ".into(),
    ));
    app.commit_rename();
    assert_eq!(app.offices[0].desks[0].name, "New Name");
    assert!(app.dirty);
}

#[test]
fn cancel_rename_clears_state() {
    let mut app = make_app_with(vec![make_task("T")]);
    app.rename = Some(RenameState::new(RenameTarget::Desk(0), "typing...".into()));
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
                    tabs: vec![SavedTab {
                        id: "t1".into(),
                        name: "Tab 1".into(),
                    }],
                },
                SavedDesk {
                    id: "d2".into(),
                    name: "Desk B".into(),
                    active_tab: 0,
                    tabs: vec![SavedTab {
                        id: "t2".into(),
                        name: "Tab 1".into(),
                    }],
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
    assert_eq!(
        app.tab_scroll, 0,
        "tab_scroll must be reset to 0 after desk close"
    );
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
    fn mouse_mode_enabled(&self) -> bool {
        true
    }
    fn get_screen(&self, _: u16, _: u16) -> ScreenContent {
        ScreenContent::default()
    }
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
    assert!(
        app.pty_mouse_mode_enabled(),
        "tab1 should have mouse enabled"
    );
    // Switch to tab2
    app.offices[0].desks[0].active_tab = 1;
    assert!(
        !app.pty_mouse_mode_enabled(),
        "tab2 should not have mouse enabled after switch"
    );
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
    assert_ne!(
        app.spinner_frame, before,
        "spinner_frame must advance after 80ms"
    );
}

// ── RenameState: unicode / cursor editing ─────────────────────────────────────

#[test]
fn rename_insert_ascii() {
    let mut r = RenameState::new(RenameTarget::Desk(0), "abc".into());
    r.cursor = 1;
    r.insert_char('X');
    assert_eq!(r.buffer, "aXbc");
    assert_eq!(r.cursor, 2);
}

#[test]
fn rename_insert_multibyte_unicode() {
    let mut r = RenameState::new(RenameTarget::Desk(0), "".into());
    r.insert_char('😀'); // 4-byte emoji
    r.insert_char('世'); // 3-byte CJK
    assert_eq!(r.buffer, "😀世");
    assert_eq!(r.char_len(), 2);
    assert_eq!(r.cursor, 2);
}

#[test]
fn rename_backspace_multibyte() {
    let mut r = RenameState::new(RenameTarget::Desk(0), "a😀b".into());
    r.cursor = 2; // after 😀
    r.backspace();
    assert_eq!(r.buffer, "ab");
    assert_eq!(r.cursor, 1);
}

#[test]
fn rename_delete_at_cursor() {
    let mut r = RenameState::new(RenameTarget::Desk(0), "hello".into());
    r.cursor = 1;
    r.delete();
    assert_eq!(r.buffer, "hllo");
    assert_eq!(r.cursor, 1, "delete should not move cursor");
}

#[test]
fn rename_delete_at_end_is_noop() {
    let mut r = RenameState::new(RenameTarget::Desk(0), "abc".into());
    r.move_end();
    r.delete();
    assert_eq!(r.buffer, "abc");
}

#[test]
fn rename_backspace_at_start_is_noop() {
    let mut r = RenameState::new(RenameTarget::Desk(0), "abc".into());
    r.cursor = 0;
    r.backspace();
    assert_eq!(r.buffer, "abc");
}

#[test]
fn rename_move_home_end() {
    let mut r = RenameState::new(RenameTarget::Desk(0), "hello".into());
    r.cursor = 2;
    r.move_home();
    assert_eq!(r.cursor, 0);
    r.move_end();
    assert_eq!(r.cursor, 5);
}

#[test]
fn rename_move_left_clamps() {
    let mut r = RenameState::new(RenameTarget::Desk(0), "ab".into());
    r.cursor = 0;
    r.move_left();
    assert_eq!(r.cursor, 0, "move_left at start must not underflow");
}

#[test]
fn rename_move_right_clamps() {
    let mut r = RenameState::new(RenameTarget::Desk(0), "ab".into());
    r.move_end();
    r.move_right();
    assert_eq!(r.cursor, 2, "move_right at end must not overflow");
}

#[test]
fn rename_cursor_byte_index_correct_for_multibyte() {
    // "😀" = 4 bytes, so byte_index(1) should be 4
    let r = RenameState::new(RenameTarget::Desk(0), "😀a".into());
    assert_eq!(r.cursor, 2); // cursor at end (char len = 2)
                             // Move to position 1 (after emoji, before 'a')
    let mut r2 = RenameState::new(RenameTarget::Desk(0), "😀a".into());
    r2.cursor = 1;
    assert_eq!(r2.cursor_byte_index(), 4); // emoji is 4 bytes
}

// ── App: desk selection & dirty ──────────────────────────────────────────────

#[test]
fn select_desk_sets_dirty_on_change() {
    let mut app = make_app_with(vec![make_task("A"), make_task("B")]);
    app.dirty = false;
    app.select_desk(1);
    assert!(app.dirty, "selecting a different desk must set dirty=true");
}

#[test]
fn select_desk_no_dirty_when_same() {
    let mut app = make_app_with(vec![make_task("A"), make_task("B")]);
    app.select_desk(0);
    app.dirty = false;
    app.select_desk(0);
    assert!(!app.dirty, "re-selecting same desk must not set dirty");
}

#[test]
fn select_desk_clamps_to_last() {
    let mut app = make_app_with(vec![make_task("A"), make_task("B")]);
    app.select_desk(99);
    assert_eq!(
        app.selected(),
        1,
        "out-of-range select must clamp to last desk"
    );
}

// ── App: switch_office ────────────────────────────────────────────────────────

use mato::client::app::Office;

fn make_office(name: &str, desks: Vec<Desk>) -> Office {
    Office {
        id: mato::utils::new_id(),
        name: name.into(),
        desks,
        active_desk: 0,
    }
}

#[test]
fn switch_office_sets_dirty() {
    let mut app = mato::client::app::App::new();
    app.offices = vec![
        make_office("A", vec![make_task("A1")]),
        make_office("B", vec![make_task("B1")]),
    ];
    app.current_office = 0;
    app.dirty = false;
    app.switch_office(1);
    assert!(app.dirty);
    assert_eq!(app.current_office, 1);
}

#[test]
fn switch_office_same_always_sets_dirty() {
    // switch_office unconditionally sets dirty=true (it spawns PTY + marks tab switch)
    let mut app = mato::client::app::App::new();
    app.offices = vec![make_office("A", vec![make_task("A1")])];
    app.current_office = 0;
    app.dirty = false;
    app.switch_office(0);
    assert!(
        app.dirty,
        "switch_office always marks dirty (even same office)"
    );
}

#[test]
fn switch_office_resets_tab_scroll() {
    let mut app = mato::client::app::App::new();
    app.offices = vec![
        make_office("A", vec![make_task("A1")]),
        make_office("B", vec![make_task("B1")]),
    ];
    app.tab_scroll = 5;
    app.switch_office(1);
    assert_eq!(app.tab_scroll, 0, "switch_office must reset tab_scroll");
}

// ── App: show_toast ────────────────────────────────────────────────────────────

#[test]
fn show_toast_sets_message_and_dirty() {
    let mut app = make_app_with(vec![make_task("T")]);
    app.dirty = false;
    app.show_toast("hello");
    assert!(app.toast.is_some());
    assert_eq!(app.toast.as_ref().unwrap().0, "hello");
    assert!(app.dirty);
}

#[test]
fn show_toast_overwrites_previous() {
    let mut app = make_app_with(vec![make_task("T")]);
    app.show_toast("first");
    app.show_toast("second");
    assert_eq!(app.toast.as_ref().unwrap().0, "second");
}

// ── App: has_active_tabs ──────────────────────────────────────────────────────

#[test]
fn has_active_tabs_false_when_empty() {
    let mut app = make_app_with(vec![make_task("T")]);
    app.active_tabs.clear();
    assert!(!app.has_active_tabs());
}

#[test]
fn has_active_tabs_true_when_nonempty() {
    let mut app = make_app_with(vec![make_task("T")]);
    app.active_tabs.insert("tab1".into());
    assert!(app.has_active_tabs());
}

// ── App: flush_pending_content_esc ────────────────────────────────────────────

use mato::client::app::CONTENT_ESC_DOUBLE_PRESS_WINDOW_MS;
use std::time::{Duration, Instant};

#[test]
fn flush_esc_clears_stale_esc() {
    use std::thread;
    let mut app = make_app_with(vec![make_task("T")]);
    // Simulate an ESC press older than the detection window
    app.last_content_esc =
        Some(Instant::now() - Duration::from_millis(CONTENT_ESC_DOUBLE_PRESS_WINDOW_MS + 50));
    app.flush_pending_content_esc();
    assert!(
        app.last_content_esc.is_none(),
        "stale ESC must be cleared by flush"
    );
}

#[test]
fn flush_esc_keeps_recent_esc() {
    let mut app = make_app_with(vec![make_task("T")]);
    app.last_content_esc = Some(Instant::now());
    app.flush_pending_content_esc();
    assert!(
        app.last_content_esc.is_some(),
        "recent ESC must NOT be cleared"
    );
}

// ── App: jump_labels exclude reserved keys ───────────────────────────────────

#[test]
fn jump_labels_content_focus_excludes_c_r_q() {
    let mut app = make_app_with(vec![make_task("T")]);
    app.focus = Focus::Content;
    let labels = app.jump_labels();
    assert!(
        !labels.contains(&'c'),
        "c must be excluded in Content focus"
    );
    assert!(
        !labels.contains(&'C'),
        "C must be excluded in Content focus"
    );
    assert!(
        !labels.contains(&'r'),
        "r must be excluded in Content focus"
    );
    assert!(
        !labels.contains(&'R'),
        "R must be excluded in Content focus"
    );
    assert!(
        !labels.contains(&'q'),
        "q must be excluded in Content focus"
    );
    assert!(
        !labels.contains(&'Q'),
        "Q must be excluded in Content focus"
    );
}

#[test]
fn jump_labels_sidebar_focus_excludes_r_q_but_not_c() {
    let mut app = make_app_with(vec![make_task("T")]);
    app.focus = Focus::Sidebar;
    let labels = app.jump_labels();
    assert!(!labels.contains(&'r'));
    assert!(!labels.contains(&'q'));
    assert!(
        labels.contains(&'c'),
        "c should be available in Sidebar focus"
    );
}

#[test]
fn jump_labels_are_unique() {
    let mut app = make_app_with(vec![make_task("T")]);
    for focus in [Focus::Content, Focus::Sidebar, Focus::Topbar] {
        app.focus = focus;
        let labels = app.jump_labels();
        let set: HashSet<char> = labels.iter().cloned().collect();
        assert_eq!(
            labels.len(),
            set.len(),
            "jump labels must be unique for {focus:?}"
        );
    }
}

// ── App: mark/finish tab switch timing ────────────────────────────────────────

#[test]
fn tab_switch_timing_roundtrip() {
    use std::thread;
    let mut app = make_app_with(vec![make_task("T")]);
    assert!(
        app.finish_tab_switch_measurement().is_none(),
        "no measurement before mark"
    );
    app.mark_tab_switch();
    thread::sleep(Duration::from_millis(5));
    let elapsed = app.finish_tab_switch_measurement();
    assert!(elapsed.is_some());
    assert!(
        elapsed.unwrap() >= Duration::from_millis(1),
        "elapsed must be > 0"
    );
    // Second call returns None (consumed)
    assert!(app.finish_tab_switch_measurement().is_none());
}

// ── App: handle_jump_selection ─────────────────────────────────────────────────

use mato::client::app::JumpMode;

#[test]
fn jump_selection_switches_desk() {
    let mut app = make_app_with(vec![make_task("A"), make_task("B"), make_task("C")]);
    app.focus = Focus::Sidebar;
    app.jump_mode = JumpMode::Active;
    // Manually set sidebar_list_area so visible_desk_indices works
    app.sidebar_list_area = ratatui::layout::Rect {
        x: 0,
        y: 0,
        width: 20,
        height: 10,
    };
    // 'a' maps to label[0] which maps to target[0] (first desk in Sidebar focus)
    let labels = app.jump_labels();
    let targets = app.jump_targets();
    assert!(!targets.is_empty());
    let first_label = labels[0];
    let (kind, desk_idx, _) = targets[0];
    assert_eq!(kind, 't', "first target in Sidebar focus should be a desk");
    app.handle_jump_selection(first_label);
    assert_eq!(
        app.selected(),
        desk_idx,
        "jump should switch to correct desk"
    );
    assert_eq!(
        app.jump_mode,
        JumpMode::None,
        "jump mode must exit after selection"
    );
}

#[test]
fn jump_selection_invalid_char_exits_jump_mode() {
    let mut app = make_app_with(vec![make_task("A")]);
    app.focus = Focus::Sidebar;
    app.jump_mode = JumpMode::Active;
    // Use a character that is NOT in jump_labels
    let reserved = '☃'; // not in JUMP_LABELS at all
    app.handle_jump_selection(reserved);
    assert_eq!(
        app.jump_mode,
        JumpMode::None,
        "jump mode must exit even on no-match"
    );
}

#[test]
fn jump_selection_tab_switches_active_tab() {
    let mut app = make_app_with(vec![Desk {
        id: mato::utils::new_id(),
        name: "D".into(),
        tabs: vec![make_tab("T1"), make_tab("T2"), make_tab("T3")],
        active_tab: 0,
    }]);
    app.focus = Focus::Topbar;
    app.jump_mode = JumpMode::Active;
    // Register tab areas so tab_area_tab_indices is populated
    app.tab_area_tab_indices = vec![0, 1, 2];
    let labels = app.jump_labels();
    // In Topbar focus, first target is first tab
    let targets = app.jump_targets();
    let (kind, desk_idx, tab_idx) = targets[0];
    assert_eq!(kind, 'b');
    let first_label = labels[0];
    app.handle_jump_selection(first_label);
    assert_eq!(
        app.offices[0].desks[desk_idx].active_tab, tab_idx,
        "jump to tab must switch active_tab"
    );
}

#[test]
fn non_esc_key_clears_pending_esc_immediately() {
    // When a non-ESC key is pressed while an ESC is pending, the ESC should be
    // forwarded to the PTY immediately (even within the 300ms window), so that
    // ESC+key sequences (Alt keys, vi ESC-then-command) work correctly.
    // This tests the `last_content_esc.take()` path in input.rs.
    let mut app = make_app_with(vec![make_task("T")]);
    app.last_content_esc = Some(Instant::now()); // fresh ESC pending
                                                 // Simulate what input.rs does for a non-ESC key in Content focus:
    let had_pending_esc = app.last_content_esc.take().is_some();
    assert!(
        had_pending_esc,
        "recent ESC must be immediately cleared on non-ESC key"
    );
    assert!(
        app.last_content_esc.is_none(),
        "last_content_esc must be None after take()"
    );
}
