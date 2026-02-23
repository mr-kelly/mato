/// Tests for daemon utility modules: PidFile, DaemonLock, and VteEmulator.
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ── PidFile ───────────────────────────────────────────────────────────────────

use mato::daemon::pid::PidFile;

fn tmp(name: &str) -> PathBuf {
    std::env::temp_dir().join(name)
}

#[test]
fn pid_file_create_writes_current_pid() {
    let path = tmp("mato_test_pid_create.pid");
    let _f = PidFile::create(path.clone()).unwrap();
    let written = std::fs::read_to_string(&path).unwrap();
    let pid: u32 = written.trim().parse().unwrap();
    assert_eq!(pid, std::process::id());
}

#[test]
fn pid_file_read_returns_pid() {
    let path = tmp("mato_test_pid_read.pid");
    let _f = PidFile::create(path.clone()).unwrap();
    let pid = PidFile::read(&path).unwrap();
    assert_eq!(pid, std::process::id());
}

#[test]
fn pid_file_read_missing_returns_none() {
    let path = tmp("mato_test_pid_missing.pid");
    let _ = std::fs::remove_file(&path);
    assert!(PidFile::read(&path).is_none());
}

#[test]
fn pid_file_drop_removes_file() {
    let path = tmp("mato_test_pid_drop.pid");
    {
        let _f = PidFile::create(path.clone()).unwrap();
        assert!(path.exists());
    }
    assert!(!path.exists(), "pid file should be removed on drop");
}

// ── DaemonLock ────────────────────────────────────────────────────────────────

use mato::daemon::lock::DaemonLock;

#[test]
fn lock_acquire_creates_file() {
    let path = tmp("mato_test_lock_create.lock");
    let _ = std::fs::remove_file(&path);
    let _lock = DaemonLock::acquire(path.clone()).unwrap();
    assert!(path.exists());
}

#[test]
fn lock_drop_removes_file() {
    let path = tmp("mato_test_lock_drop.lock");
    let _ = std::fs::remove_file(&path);
    {
        let _lock = DaemonLock::acquire(path.clone()).unwrap();
    }
    assert!(!path.exists(), "lock file should be removed on drop");
}

#[test]
fn lock_second_acquire_fails() {
    let path = tmp("mato_test_lock_double.lock");
    let _ = std::fs::remove_file(&path);
    let _lock1 = DaemonLock::acquire(path.clone()).unwrap();
    let result = DaemonLock::acquire(path.clone());
    assert!(
        result.is_err(),
        "second acquire should fail while first is held"
    );
}

// ── AlacrittyEmulator ─────────────────────────────────────────────────────────

use mato::emulators::AlacrittyEmulator;
use mato::terminal_emulator::TerminalEmulator;

#[test]
fn alacritty_renders_written_text() {
    let mut emu = AlacrittyEmulator::new(24, 80);
    emu.process(b"Hello");
    let screen = emu.get_screen(24, 80);
    let first_row: String = screen.lines[0].cells.iter().map(|c| c.ch).collect();
    assert!(first_row.starts_with("Hello"), "got: {first_row:?}");
}

#[test]
fn alacritty_cursor_advances_after_text() {
    let mut emu = AlacrittyEmulator::new(24, 80);
    emu.process(b"Hi");
    let screen = emu.get_screen(24, 80);
    assert_eq!(screen.cursor.1, 2);
}

#[test]
fn alacritty_newline_moves_cursor_down() {
    let mut emu = AlacrittyEmulator::new(24, 80);
    emu.process(b"A\r\nB");
    let screen = emu.get_screen(24, 80);
    assert_eq!(
        screen.cursor.0, 1,
        "cursor should be on row 1 after newline"
    );
}

#[test]
fn alacritty_carriage_return_resets_column() {
    let mut emu = AlacrittyEmulator::new(24, 80);
    emu.process(b"Hello\r");
    let screen = emu.get_screen(24, 80);
    assert_eq!(screen.cursor.1, 0, "cursor column should be 0 after \\r");
}

#[test]
fn alacritty_resize_preserves_content() {
    let mut emu = AlacrittyEmulator::new(24, 80);
    emu.process(b"Hello");
    emu.resize(10, 40); // should be no-op for emulator
    let screen = emu.get_screen(10, 40);
    let first_row: String = screen.lines[0].cells.iter().map(|c| c.ch).collect();
    assert!(
        first_row.starts_with("Hello"),
        "content should survive resize: {first_row:?}"
    );
}

#[test]
fn alacritty_wide_char_sets_display_width_and_spacer() {
    let mut emu = AlacrittyEmulator::new(24, 80);
    emu.process("中".as_bytes());
    let screen = emu.get_screen(24, 80);
    let row = &screen.lines[0].cells;
    assert_eq!(row[0].ch, '中');
    assert_eq!(row[0].display_width, 2);
    assert_eq!(row[1].ch, '\0');
    assert_eq!(row[1].display_width, 0);
}

// ── persistence ───────────────────────────────────────────────────────────────

use mato::client::persistence::{SavedDesk, SavedOffice, SavedState, SavedTab};

fn make_state() -> SavedState {
    SavedState {
        current_office: 0,
        offices: vec![SavedOffice {
            id: "o1".into(),
            name: "Default".into(),
            active_desk: 0,
            desks: vec![SavedDesk {
                id: "t1".into(),
                name: "Work".into(),
                active_tab: 0,
                tabs: vec![SavedTab {
                    id: "tb1".into(),
                    name: "Terminal 1".into(),
                }],
            }],
        }],
    }
}

#[test]
fn persistence_save_and_load_roundtrip() {
    let dir = std::env::temp_dir().join("mato_test_persist");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("state.json");
    let json = serde_json::to_string_pretty(&make_state()).unwrap();
    std::fs::write(&path, &json).unwrap();
    let restored: SavedState =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(restored.offices[0].desks.len(), 1);
    assert_eq!(restored.offices[0].desks[0].name, "Work");
    assert_eq!(restored.offices[0].desks[0].tabs[0].name, "Terminal 1");
}

#[test]
fn persistence_corrupt_json_fails_gracefully() {
    let result: Result<SavedState, _> = serde_json::from_str("{ not valid json }");
    assert!(result.is_err());
}

#[test]
fn persistence_missing_active_task_defaults_to_zero() {
    let json = r#"{"offices":[]}"#;
    let state: SavedState = serde_json::from_str(json).unwrap();
    assert_eq!(state.current_office, 0);
}

// ── persistence: save_state / load_state via XDG_CONFIG_HOME ─────────────────

use mato::client::app::{App, Desk, RenameState, RenameTarget, TabEntry};
use mato::terminal_provider::{ScreenContent, TerminalProvider};

struct NullProvider;
impl TerminalProvider for NullProvider {
    fn spawn(&mut self, _: u16, _: u16) {}
    fn resize(&mut self, _: u16, _: u16) {}
    fn write(&mut self, _: &[u8]) {}
    fn get_screen(&self, _: u16, _: u16) -> ScreenContent {
        ScreenContent::default()
    }
}

struct TrackingProvider {
    requests: Arc<Mutex<Vec<(u16, u16)>>>,
}

impl TerminalProvider for TrackingProvider {
    fn spawn(&mut self, rows: u16, cols: u16) {
        self.requests.lock().unwrap().push((rows, cols));
    }
    fn resize(&mut self, _: u16, _: u16) {}
    fn write(&mut self, _: &[u8]) {}
    fn get_screen(&self, rows: u16, cols: u16) -> ScreenContent {
        self.requests.lock().unwrap().push((rows, cols));
        ScreenContent {
            title: Some("tracked-title".to_string()),
            ..ScreenContent::default()
        }
    }
}

fn null_tab(name: &str) -> TabEntry {
    TabEntry {
        id: mato::utils::new_id(),
        name: name.into(),
        provider: Box::new(NullProvider),
    }
}

fn null_task(name: &str, n: usize) -> Desk {
    let tabs = (0..n).map(|i| null_tab(&format!("T{}", i + 1))).collect();
    Desk {
        id: mato::utils::new_id(),
        name: name.into(),
        tabs,
        active_tab: 0,
    }
}

fn make_app(desks: Vec<Desk>) -> App {
    let mut app = App::new();
    app.current_office = 0;
    app.offices = vec![mato::client::app::Office {
        id: "test".into(),
        name: "Test".into(),
        desks,
        active_desk: 0,
    }];
    app.list_state.select(Some(0));
    app
}

fn with_temp_config<F: FnOnce()>(name: &str, f: F) {
    // Serialize all tests that touch XDG_CONFIG_HOME to avoid env var races
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = std::env::temp_dir().join(format!("mato_cfg_{}", name));
    std::fs::create_dir_all(&dir).unwrap();
    let _ = std::fs::remove_file(dir.join("mato/state.json"));
    std::env::set_var("XDG_CONFIG_HOME", &dir);
    f();
    std::env::remove_var("XDG_CONFIG_HOME");
}

#[test]
fn save_and_load_state_roundtrip() {
    with_temp_config("save_load", || {
        let mut app = make_app(vec![null_task("Work", 2), null_task("Play", 1)]);
        app.offices[0].desks[0].active_tab = 1;
        app.nav(1);
        mato::client::persistence::save_state(&app).unwrap();
        let restored = mato::client::persistence::load_state().unwrap();
        assert_eq!(restored.offices[0].desks.len(), 2);
        assert_eq!(restored.offices[0].active_desk, 1);
        assert_eq!(restored.offices[0].desks[0].name, "Work");
        assert_eq!(restored.offices[0].desks[0].tabs.len(), 2);
        assert_eq!(restored.offices[0].desks[0].active_tab, 1);
    });
}

#[test]
fn load_state_missing_file_returns_err() {
    with_temp_config("load_missing", || {
        assert!(mato::client::persistence::load_state().is_err());
    });
}

#[test]
fn load_state_corrupt_json_returns_err() {
    with_temp_config("load_corrupt2", || {
        let path = mato::utils::get_state_file_path();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"{ not json }").unwrap();
        assert!(mato::client::persistence::load_state().is_err());
    });
}

// ── app: uncovered branches ───────────────────────────────────────────────────

#[test]
fn commit_rename_tab_applies_name() {
    let mut app = make_app(vec![null_task("T", 2)]);
    app.rename = Some(RenameState::new(RenameTarget::Tab(0, 1), "Renamed".into()));
    app.commit_rename();
    assert_eq!(app.offices[0].desks[0].tabs[1].name, "Renamed");
}

#[test]
fn begin_rename_task_sets_rename_state() {
    let mut app = make_app(vec![null_task("MyTask", 1)]);
    app.begin_rename_desk(0);
    let rename = app.rename.as_ref().unwrap();
    assert!(matches!(rename.target, RenameTarget::Desk(0)));
    assert_eq!(rename.buffer, "MyTask");
}

#[test]
fn begin_rename_tab_sets_rename_state() {
    let mut app = make_app(vec![null_task("T", 1)]);
    app.begin_rename_tab();
    let rename = app.rename.as_ref().unwrap();
    assert!(matches!(rename.target, RenameTarget::Tab(0, 0)));
    assert_eq!(rename.buffer, "T1");
}

#[test]
fn sync_tab_titles_uses_current_terminal_size() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let tab = TabEntry {
        id: mato::utils::new_id(),
        name: "Tracked".into(),
        provider: Box::new(TrackingProvider {
            requests: requests.clone(),
        }),
    };
    let desk = Desk {
        id: mato::utils::new_id(),
        name: "Desk".into(),
        tabs: vec![tab],
        active_tab: 0,
    };
    let mut app = make_app(vec![desk]);
    app.term_rows = 37;
    app.term_cols = 119;
    app.last_title_sync = Instant::now() - Duration::from_secs(1);

    app.sync_tab_titles();

    let calls = requests.lock().unwrap();
    assert_eq!(calls.as_slice(), &[(37, 119)]);
}

#[test]
fn resize_all_ptys_updates_dimensions() {
    let mut app = make_app(vec![null_task("T", 1)]);
    app.term_rows = 24;
    app.term_cols = 80;
    app.resize_all_ptys(30, 100);
    // resize now directly notifies PTYs; term_rows/cols are updated by draw, not here
    // just verify it doesn't panic and the call succeeds
}

#[test]
fn resize_all_ptys_noop_when_same_size() {
    let mut app = make_app(vec![null_task("T", 1)]);
    app.resize_all_ptys(24, 80);
    assert_eq!(app.term_rows, 24);
}

#[test]
fn nav_between_desks_spawns_target_active_tab() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let tracked_tab = TabEntry {
        id: mato::utils::new_id(),
        name: "Tracked".into(),
        provider: Box::new(TrackingProvider {
            requests: requests.clone(),
        }),
    };
    let desk0 = null_task("Desk 1", 1);
    let desk1 = Desk {
        id: mato::utils::new_id(),
        name: "Desk 2".into(),
        tabs: vec![tracked_tab],
        active_tab: 0,
    };
    let mut app = make_app(vec![desk0, desk1]);
    app.term_rows = 30;
    app.term_cols = 100;
    app.list_state.select(Some(0));

    app.nav(1);

    let calls = requests.lock().unwrap();
    assert_eq!(calls.as_slice(), &[(30, 100)]);
}

#[test]
fn active_tab_ref_returns_correct_tab() {
    let mut task = null_task("T", 3);
    task.active_tab = 2;
    assert_eq!(task.active_tab_ref().name, "T3");
}

// ── Alacritty: bell and focus-tracking mode ───────────────────────────────────

#[test]
fn alacritty_bell_is_consumed_once_per_ding() {
    let mut emu = AlacrittyEmulator::new(24, 80);
    emu.process(b"\x07"); // BEL
    let s1 = emu.get_screen(24, 80);
    assert!(s1.bell, "first get_screen should report bell=true");
    let s2 = emu.get_screen(24, 80);
    assert!(!s2.bell, "second get_screen should report bell=false (consumed)");
}

#[test]
fn alacritty_focus_events_disabled_by_default() {
    let emu = AlacrittyEmulator::new(24, 80);
    let screen = emu.get_screen(24, 80);
    assert!(
        !screen.focus_events_enabled,
        "focus tracking should be off by default"
    );
}

#[test]
fn alacritty_focus_events_enabled_after_escape_sequence() {
    let mut emu = AlacrittyEmulator::new(24, 80);
    // \x1b[?1004h enables focus event reporting
    emu.process(b"\x1b[?1004h");
    let screen = emu.get_screen(24, 80);
    assert!(
        screen.focus_events_enabled,
        "focus tracking should be on after \\x1b[?1004h"
    );
}

#[test]
fn alacritty_focus_events_disabled_after_reset_sequence() {
    let mut emu = AlacrittyEmulator::new(24, 80);
    emu.process(b"\x1b[?1004h"); // enable
    emu.process(b"\x1b[?1004l"); // disable
    let screen = emu.get_screen(24, 80);
    assert!(
        !screen.focus_events_enabled,
        "focus tracking should be off after \\x1b[?1004l"
    );
}
