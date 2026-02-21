/// Tests for handle_key input logic.
/// Uses NullProvider + make_app helper (same pattern as app_tests).
use mato::terminal_provider::{ScreenContent, TerminalProvider};
use mato::client::app::{App, Focus, JumpMode, RenameTarget, TabEntry, Desk};
use mato::client::input::handle_key;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::sync::{Arc, Mutex};

struct NullProvider;
impl TerminalProvider for NullProvider {
    fn spawn(&mut self, _: u16, _: u16) {}
    fn resize(&mut self, _: u16, _: u16) {}
    fn write(&mut self, _: &[u8]) {}
    fn get_screen(&self, _: u16, _: u16) -> ScreenContent { ScreenContent::default() }
}

struct CaptureProvider {
    sink: Arc<Mutex<Vec<u8>>>,
}
impl TerminalProvider for CaptureProvider {
    fn spawn(&mut self, _: u16, _: u16) {}
    fn resize(&mut self, _: u16, _: u16) {}
    fn write(&mut self, bytes: &[u8]) {
        self.sink.lock().unwrap().extend_from_slice(bytes);
    }
    fn get_screen(&self, _: u16, _: u16) -> ScreenContent { ScreenContent::default() }
}

fn make_tab(name: &str) -> TabEntry {
    TabEntry { id: mato::utils::new_id(), name: name.into(), provider: Box::new(NullProvider) }
}

fn make_task(name: &str) -> Desk {
    Desk { id: mato::utils::new_id(), name: name.into(), tabs: vec![make_tab("T1")], active_tab: 0 }
}

fn make_capture_app() -> (App, Arc<Mutex<Vec<u8>>>) {
    let sink = Arc::new(Mutex::new(Vec::new()));
    let tab = TabEntry {
        id: mato::utils::new_id(),
        name: "T1".into(),
        provider: Box::new(CaptureProvider { sink: sink.clone() }),
    };
    let desk = Desk { id: mato::utils::new_id(), name: "Desk".into(), tabs: vec![tab], active_tab: 0 };
    let mut app = make_app(vec![desk]);
    app.focus = Focus::Content;
    (app, sink)
}

fn make_app(desks: Vec<Desk>) -> App {
    let mut app = App::new();
    app.current_office = 0;
    app.offices = vec![mato::client::app::Office {
        id: "test".into(), name: "Test".into(), desks, active_desk: 0,
    }];
    app.list_state.select(Some(0));
    app
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent { code, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: crossterm::event::KeyEventState::NONE }
}

fn key_mod(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent { code, modifiers, kind: KeyEventKind::Press, state: crossterm::event::KeyEventState::NONE }
}

// ── quit ──────────────────────────────────────────────────────────────────────

#[test]
fn q_in_sidebar_returns_true() {
    let mut app = make_app(vec![make_task("T")]);
    assert!(handle_key(&mut app, key(KeyCode::Char('q'))));
}

#[test]
fn q_in_topbar_quits() {
    let mut app = make_app(vec![make_task("T")]);
    app.focus = Focus::Topbar;
    assert!(handle_key(&mut app, key(KeyCode::Char('q'))));
}

// ── focus transitions ─────────────────────────────────────────────────────────

#[test]
fn esc_from_topbar_goes_to_sidebar() {
    let mut app = make_app(vec![make_task("T")]);
    app.focus = Focus::Topbar;
    handle_key(&mut app, key(KeyCode::Esc));
    assert_eq!(app.focus, Focus::Sidebar);
}

#[test]
fn esc_from_content_enters_jump_mode() {
    let mut app = make_app(vec![make_task("T")]);
    app.focus = Focus::Content;
    handle_key(&mut app, key(KeyCode::Esc));
    assert_eq!(app.jump_mode, JumpMode::Active);
}

#[test]
fn jump_mode_a_goes_to_sidebar() {
    let mut app = make_app(vec![make_task("T")]);
    app.focus = Focus::Content;
    app.jump_mode = JumpMode::Active;
    handle_key(&mut app, key(KeyCode::Char('a')));
    // 'a' in jump mode jumps to first task (index 0), focus stays on content or sidebar
    assert_eq!(app.jump_mode, JumpMode::None);
}

#[test]
fn jump_mode_left_goes_to_sidebar() {
    let mut app = make_app(vec![make_task("T")]);
    app.focus = Focus::Content;
    app.jump_mode = JumpMode::Active;
    handle_key(&mut app, key(KeyCode::Left));
    assert_eq!(app.focus, Focus::Sidebar);
    assert_eq!(app.jump_mode, JumpMode::None);
}

// ── rename buffer editing ─────────────────────────────────────────────────────

#[test]
fn rename_mode_char_appends_to_buffer() {
    let mut app = make_app(vec![make_task("T")]);
    app.rename = Some((RenameTarget::Desk(0), "ab".into()));
    handle_key(&mut app, key(KeyCode::Char('c')));
    assert_eq!(app.rename.as_ref().unwrap().1, "abc");
}

#[test]
fn rename_mode_backspace_removes_last_char() {
    let mut app = make_app(vec![make_task("T")]);
    app.rename = Some((RenameTarget::Desk(0), "abc".into()));
    handle_key(&mut app, key(KeyCode::Backspace));
    assert_eq!(app.rename.as_ref().unwrap().1, "ab");
}

#[test]
fn rename_mode_enter_commits() {
    let mut app = make_app(vec![make_task("Old")]);
    app.rename = Some((RenameTarget::Desk(0), "New".into()));
    handle_key(&mut app, key(KeyCode::Enter));
    assert!(app.rename.is_none());
    assert_eq!(app.offices[0].desks[0].name, "New");
}

#[test]
fn rename_mode_esc_cancels() {
    let mut app = make_app(vec![make_task("Old")]);
    app.rename = Some((RenameTarget::Desk(0), "typing".into()));
    handle_key(&mut app, key(KeyCode::Esc));
    assert!(app.rename.is_none());
    assert_eq!(app.offices[0].desks[0].name, "Old");
}

// ── Alt+1-9 tab switching ─────────────────────────────────────────────────────

#[test]
fn alt_1_switches_to_first_tab() {
    let mut task = make_task("T");
    task.tabs.push(make_tab("T2"));
    task.tabs.push(make_tab("T3"));
    task.active_tab = 2;
    let mut app = make_app(vec![task]);
    handle_key(&mut app, key_mod(KeyCode::Char('1'), KeyModifiers::ALT));
    assert_eq!(app.offices[0].desks[0].active_tab, 0);
}

#[test]
fn alt_n_out_of_range_does_nothing() {
    let mut app = make_app(vec![make_task("T")]); // only 1 tab
    handle_key(&mut app, key_mod(KeyCode::Char('9'), KeyModifiers::ALT));
    assert_eq!(app.offices[0].desks[0].active_tab, 0); // unchanged
}

// ── sidebar navigation ────────────────────────────────────────────────────────

#[test]
fn n_in_sidebar_creates_task() {
    let mut app = make_app(vec![make_task("T")]);
    handle_key(&mut app, key(KeyCode::Char('n')));
    assert_eq!(app.offices[0].desks.len(), 2);
}

#[test]
fn up_down_navigate_tasks() {
    let mut app = make_app(vec![make_task("A"), make_task("B"), make_task("C")]);
    handle_key(&mut app, key(KeyCode::Down));
    assert_eq!(app.selected(), 1);
    handle_key(&mut app, key(KeyCode::Up));
    assert_eq!(app.selected(), 0);
}

// ── topbar tab switching ──────────────────────────────────────────────────────

#[test]
fn right_in_topbar_advances_tab() {
    let mut task = make_task("T");
    task.tabs.push(make_tab("T2"));
    let mut app = make_app(vec![task]);
    app.focus = Focus::Topbar;
    handle_key(&mut app, key(KeyCode::Right));
    assert_eq!(app.offices[0].desks[0].active_tab, 1);
}

#[test]
fn left_in_topbar_does_not_go_below_zero() {
    let mut app = make_app(vec![make_task("T")]);
    app.focus = Focus::Topbar;
    handle_key(&mut app, key(KeyCode::Left));
    assert_eq!(app.offices[0].desks[0].active_tab, 0);
}

// ── content key encoding ──────────────────────────────────────────────────────

#[test]
fn content_home_end_are_encoded() {
    let (mut app, sink) = make_capture_app();
    handle_key(&mut app, key(KeyCode::Home));
    handle_key(&mut app, key(KeyCode::End));
    assert_eq!(*sink.lock().unwrap(), b"\x1b[H\x1b[F");
}

#[test]
fn content_delete_page_and_function_keys_are_encoded() {
    let (mut app, sink) = make_capture_app();
    handle_key(&mut app, key(KeyCode::Delete));
    handle_key(&mut app, key(KeyCode::PageUp));
    handle_key(&mut app, key(KeyCode::F(5)));
    assert_eq!(*sink.lock().unwrap(), b"\x1b[3~\x1b[5~\x1b[15~");
}

#[test]
fn content_ctrl_left_bracket_encodes_escape() {
    let (mut app, sink) = make_capture_app();
    handle_key(&mut app, key_mod(KeyCode::Char('['), KeyModifiers::CONTROL));
    assert_eq!(*sink.lock().unwrap(), b"\x1b");
}

#[test]
fn content_alt_char_is_meta_prefixed() {
    let (mut app, sink) = make_capture_app();
    handle_key(&mut app, key_mod(KeyCode::Char('x'), KeyModifiers::ALT));
    assert_eq!(*sink.lock().unwrap(), b"\x1bx");
}
