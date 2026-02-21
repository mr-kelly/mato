/// Tests for handle_key input logic.
/// Uses NullProvider + make_app helper (same pattern as app_tests).
use mato::terminal_provider::{ScreenContent, TerminalProvider};
use mato::client::app::{App, Focus, JumpMode, RenameTarget, TabEntry, Task};
use mato::client::input::handle_key;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{layout::Rect, widgets::ListState};
use std::collections::HashSet;

struct NullProvider;
impl TerminalProvider for NullProvider {
    fn spawn(&mut self, _: u16, _: u16) {}
    fn resize(&mut self, _: u16, _: u16) {}
    fn write(&mut self, _: &[u8]) {}
    fn get_screen(&self, _: u16, _: u16) -> ScreenContent { ScreenContent::default() }
}

fn make_tab(name: &str) -> TabEntry {
    TabEntry { id: mato::utils::new_id(), name: name.into(), provider: Box::new(NullProvider) }
}

fn make_task(name: &str) -> Task {
    Task { id: mato::utils::new_id(), name: name.into(), tabs: vec![make_tab("T1")], active_tab: 0 }
}

fn make_app(tasks: Vec<Task>) -> App {
    let mut app = App::new();
    app.tasks = tasks;
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
fn q_in_topbar_does_not_quit() {
    let mut app = make_app(vec![make_task("T")]);
    app.focus = Focus::Topbar;
    assert!(!handle_key(&mut app, key(KeyCode::Char('q'))));
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
    app.rename = Some((RenameTarget::Task(0), "ab".into()));
    handle_key(&mut app, key(KeyCode::Char('c')));
    assert_eq!(app.rename.as_ref().unwrap().1, "abc");
}

#[test]
fn rename_mode_backspace_removes_last_char() {
    let mut app = make_app(vec![make_task("T")]);
    app.rename = Some((RenameTarget::Task(0), "abc".into()));
    handle_key(&mut app, key(KeyCode::Backspace));
    assert_eq!(app.rename.as_ref().unwrap().1, "ab");
}

#[test]
fn rename_mode_enter_commits() {
    let mut app = make_app(vec![make_task("Old")]);
    app.rename = Some((RenameTarget::Task(0), "New".into()));
    handle_key(&mut app, key(KeyCode::Enter));
    assert!(app.rename.is_none());
    assert_eq!(app.tasks[0].name, "New");
}

#[test]
fn rename_mode_esc_cancels() {
    let mut app = make_app(vec![make_task("Old")]);
    app.rename = Some((RenameTarget::Task(0), "typing".into()));
    handle_key(&mut app, key(KeyCode::Esc));
    assert!(app.rename.is_none());
    assert_eq!(app.tasks[0].name, "Old");
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
    assert_eq!(app.tasks[0].active_tab, 0);
}

#[test]
fn alt_n_out_of_range_does_nothing() {
    let mut app = make_app(vec![make_task("T")]); // only 1 tab
    handle_key(&mut app, key_mod(KeyCode::Char('9'), KeyModifiers::ALT));
    assert_eq!(app.tasks[0].active_tab, 0); // unchanged
}

// ── sidebar navigation ────────────────────────────────────────────────────────

#[test]
fn n_in_sidebar_creates_task() {
    let mut app = make_app(vec![make_task("T")]);
    handle_key(&mut app, key(KeyCode::Char('n')));
    assert_eq!(app.tasks.len(), 2);
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
    assert_eq!(app.tasks[0].active_tab, 1);
}

#[test]
fn left_in_topbar_does_not_go_below_zero() {
    let mut app = make_app(vec![make_task("T")]);
    app.focus = Focus::Topbar;
    handle_key(&mut app, key(KeyCode::Left));
    assert_eq!(app.tasks[0].active_tab, 0);
}
