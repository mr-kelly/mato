/// Tests for daemon utility modules: PidFile, DaemonLock, and VteEmulator.
use std::path::PathBuf;

// ── PidFile ───────────────────────────────────────────────────────────────────

use mato::daemon_modules::pid::PidFile;

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

use mato::daemon_modules::lock::DaemonLock;

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
    assert!(result.is_err(), "second acquire should fail while first is held");
}

// ── VteEmulator ───────────────────────────────────────────────────────────────

use mato::terminal_emulator::TerminalEmulator;
use mato::emulators::VteEmulator;

#[test]
fn vte_renders_written_text() {
    let mut emu = VteEmulator::new(24, 80);
    emu.process(b"Hello");
    let screen = emu.get_screen(24, 80);
    let first_row: String = screen.lines[0].cells.iter().map(|c| c.ch).collect();
    assert!(first_row.starts_with("Hello"), "got: {first_row:?}");
}

#[test]
fn vte_cursor_advances_after_text() {
    let mut emu = VteEmulator::new(24, 80);
    emu.process(b"Hi");
    let screen = emu.get_screen(24, 80);
    assert_eq!(screen.cursor.1, 2);
}

#[test]
fn vte_newline_moves_cursor_down() {
    let mut emu = VteEmulator::new(24, 80);
    emu.process(b"A\nB");
    let screen = emu.get_screen(24, 80);
    assert_eq!(screen.cursor.0, 1, "cursor should be on row 1 after \\n");
}

#[test]
fn vte_carriage_return_resets_column() {
    let mut emu = VteEmulator::new(24, 80);
    emu.process(b"Hello\r");
    let screen = emu.get_screen(24, 80);
    assert_eq!(screen.cursor.1, 0, "cursor column should be 0 after \\r");
}

#[test]
fn vte_resize_clears_screen() {
    let mut emu = VteEmulator::new(24, 80);
    emu.process(b"Hello");
    emu.resize(10, 40);
    let screen = emu.get_screen(10, 40);
    let first_row: String = screen.lines[0].cells.iter().map(|c| c.ch).collect();
    assert!(first_row.chars().all(|c| c == ' '), "screen should be blank after resize");
}

// ── persistence ───────────────────────────────────────────────────────────────

use mato::client::persistence::{SavedState, SavedTab, SavedTask};

fn make_state() -> SavedState {
    SavedState {
        active_task: 0,
        tasks: vec![SavedTask {
            id: "t1".into(), name: "Work".into(), active_tab: 0,
            tabs: vec![SavedTab { id: "tb1".into(), name: "Terminal 1".into() }],
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
    let restored: SavedState = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(restored.tasks.len(), 1);
    assert_eq!(restored.tasks[0].name, "Work");
    assert_eq!(restored.tasks[0].tabs[0].name, "Terminal 1");
}

#[test]
fn persistence_corrupt_json_fails_gracefully() {
    let result: Result<SavedState, _> = serde_json::from_str("{ not valid json }");
    assert!(result.is_err());
}

#[test]
fn persistence_missing_active_task_defaults_to_zero() {
    let json = r#"{"tasks":[]}"#;
    let state: SavedState = serde_json::from_str(json).unwrap();
    assert_eq!(state.active_task, 0);
}

// ── persistence: save_state / load_state via XDG_CONFIG_HOME ─────────────────

use mato::terminal_provider::{ScreenContent, TerminalProvider};
use mato::client::app::{App, Focus, RenameTarget, TabEntry, Task};
use ratatui::widgets::ListState;
use std::collections::HashSet;

struct NullProvider;
impl TerminalProvider for NullProvider {
    fn spawn(&mut self, _: u16, _: u16) {}
    fn resize(&mut self, _: u16, _: u16) {}
    fn write(&mut self, _: &[u8]) {}
    fn get_screen(&self, _: u16, _: u16) -> ScreenContent { ScreenContent::default() }
}

fn null_tab(name: &str) -> TabEntry {
    TabEntry { id: mato::utils::new_id(), name: name.into(), provider: Box::new(NullProvider) }
}

fn null_task(name: &str, n: usize) -> Task {
    let tabs = (0..n).map(|i| null_tab(&format!("T{}", i + 1))).collect();
    Task { id: mato::utils::new_id(), name: name.into(), tabs, active_tab: 0 }
}

fn make_app(tasks: Vec<Task>) -> App {
    let mut app = App::new();
    app.tasks = tasks;
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
        app.tasks[0].active_tab = 1;
        mato::client::persistence::save_state(&app).unwrap();
        let restored = mato::client::persistence::load_state().unwrap();
        assert_eq!(restored.tasks.len(), 2);
        assert_eq!(restored.tasks[0].name, "Work");
        assert_eq!(restored.tasks[0].tabs.len(), 2);
        assert_eq!(restored.tasks[0].active_tab, 1);
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
    app.rename = Some((RenameTarget::Tab(0, 1), "Renamed".into()));
    app.commit_rename();
    assert_eq!(app.tasks[0].tabs[1].name, "Renamed");
}

#[test]
fn begin_rename_task_sets_rename_state() {
    let mut app = make_app(vec![null_task("MyTask", 1)]);
    app.begin_rename_task(0);
    let (target, buf) = app.rename.as_ref().unwrap();
    assert!(matches!(target, RenameTarget::Task(0)));
    assert_eq!(buf, "MyTask");
}

#[test]
fn begin_rename_tab_sets_rename_state() {
    let mut app = make_app(vec![null_task("T", 1)]);
    app.begin_rename_tab();
    let (target, buf) = app.rename.as_ref().unwrap();
    assert!(matches!(target, RenameTarget::Tab(0, 0)));
    assert_eq!(buf, "T1");
}

#[test]
fn resize_all_ptys_updates_dimensions() {
    let mut app = make_app(vec![null_task("T", 1)]);
    app.resize_all_ptys(30, 100);
    // resize is deferred; pending_resize should be set
    assert!(app.pending_resize.is_some());
    let (r, c, _) = app.pending_resize.unwrap();
    assert_eq!(r, 30);
    assert_eq!(c, 100);
}

#[test]
fn resize_all_ptys_noop_when_same_size() {
    let mut app = make_app(vec![null_task("T", 1)]);
    app.resize_all_ptys(24, 80);
    assert_eq!(app.term_rows, 24);
}

#[test]
fn active_tab_ref_returns_correct_tab() {
    let mut task = null_task("T", 3);
    task.active_tab = 2;
    assert_eq!(task.active_tab_ref().name, "T3");
}
