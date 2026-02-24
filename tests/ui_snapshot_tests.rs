/// Snapshot tests for Mato UI rendering using ratatui TestBackend + insta.
/// Run with: cargo test --test ui_snapshot_tests
/// Update snapshots: cargo insta review
use mato::client::app::{App, Desk, Focus, Office, TabEntry};
use mato::client::ui::draw;
use mato::terminal_provider::{ScreenContent, TerminalProvider};
use ratatui::{backend::TestBackend, Terminal};

struct NullProvider;
impl TerminalProvider for NullProvider {
    fn spawn(&mut self, _: u16, _: u16) {}
    fn resize(&mut self, _: u16, _: u16) {}
    fn write(&mut self, _: &[u8]) {}
    fn get_screen(&self, _: u16, _: u16) -> ScreenContent {
        ScreenContent::default()
    }
}

fn make_tab(name: &str) -> TabEntry {
    TabEntry {
        id: format!("tab-{}", name.to_lowercase().replace(' ', "-")),
        name: name.into(),
        provider: Box::new(NullProvider),
    }
}

fn make_desk(name: &str, tabs: Vec<&str>) -> Desk {
    Desk {
        id: format!("desk-{}", name.to_lowercase().replace(' ', "-")),
        name: name.into(),
        tabs: tabs.iter().map(|t| make_tab(t)).collect(),
        active_tab: 0,
    }
}

fn make_app(desks: Vec<Desk>) -> App {
    let mut app = App::new();
    // Snapshot tests must be deterministic across local COLORTERM/theme settings.
    app.toast = Some((
        "Theme disabled: terminal lacks truecolor (set COLORTERM=truecolor)".into(),
        std::time::Instant::now(),
    ));
    app.current_office = 0;
    app.offices = vec![Office {
        id: "office-test".into(),
        name: "Test".into(),
        desks,
        active_desk: 0,
    }];
    app.list_state.select(Some(0));
    app.term_rows = 24;
    app.term_cols = 90;
    app
}

fn render(app: &mut App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| draw(f, app)).unwrap();
    format!("{}", terminal.backend())
}

// ── Sidebar + topbar layout ────────────────────────────────────────────────

#[test]
fn snapshot_single_desk_single_tab() {
    let mut app = make_app(vec![make_desk("Code", vec!["Terminal 1"])]);
    app.focus = Focus::Content;
    let out = render(&mut app, 100, 30);
    insta::assert_snapshot!("single_desk_single_tab", out);
}

#[test]
fn snapshot_multiple_desks() {
    let mut app = make_app(vec![
        make_desk("Code", vec!["Terminal 1", "Terminal 2"]),
        make_desk("Design Lab", vec!["Terminal 1"]),
        make_desk("Infra", vec!["Terminal 1"]),
    ]);
    app.focus = Focus::Sidebar;
    let out = render(&mut app, 100, 30);
    insta::assert_snapshot!("multiple_desks", out);
}

#[test]
fn snapshot_topbar_many_tabs() {
    // Many tabs to trigger tab_scroll > 0 in some configurations
    let tabs = vec!["Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta"];
    let mut app = make_app(vec![make_desk("Work", tabs)]);
    app.focus = Focus::Topbar;
    let out = render(&mut app, 100, 30);
    insta::assert_snapshot!("topbar_many_tabs", out);
}

#[test]
fn snapshot_active_desk_highlighted() {
    let mut app = make_app(vec![
        make_desk("Active", vec!["Terminal 1"]),
        make_desk("Other", vec!["Terminal 1"]),
    ]);
    // Select second desk
    app.select_desk(1);
    app.focus = Focus::Content;
    let out = render(&mut app, 100, 30);
    insta::assert_snapshot!("second_desk_selected", out);
}

#[test]
fn snapshot_narrow_terminal() {
    // Narrow terminal triggers minimal layout (no sidebar)
    let mut app = make_app(vec![make_desk("Narrow", vec!["Terminal 1"])]);
    app.focus = Focus::Content;
    let out = render(&mut app, 50, 20);
    insta::assert_snapshot!("narrow_no_sidebar", out);
}

// ── Rename popup ───────────────────────────────────────────────────────────

#[test]
fn snapshot_rename_popup_visible() {
    let mut app = make_app(vec![make_desk("OldName", vec!["Terminal 1"])]);
    app.begin_rename_tab();
    let out = render(&mut app, 100, 30);
    insta::assert_snapshot!("rename_popup_visible", out);
}
