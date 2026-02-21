mod app;
mod id;
mod persistence;
mod ui;
mod input;
mod terminal_provider;
mod pty_provider;

use std::{io, time::Duration};

#[cfg(unix)]
extern crate libc;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use app::{App, Focus};
use persistence::save_state;
use ui::draw;
use input::handle_key;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App::new();
    terminal.draw(|f| draw(f, &mut app))?;
    app.spawn_active_pty();

    loop {
        terminal.draw(|f| draw(f, &mut app))?;
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) => {
                    if handle_key(&mut app, key) { break; }
                    if app.dirty { save_state(&app); app.dirty = false; }
                }
                Event::Mouse(me) => handle_mouse(&mut app, me),
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

fn handle_mouse(app: &mut App, me: crossterm::event::MouseEvent) {
    let (col, row) = (me.column, me.row);
    fn in_rect(col: u16, row: u16, r: Rect) -> bool {
        col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height
    }
    match me.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let mut handled = false;

            // New Task button
            if in_rect(col, row, app.new_task_area) {
                app.new_task(); app.focus = Focus::Sidebar; handled = true;
            }

            // Sidebar task list
            let a = app.sidebar_list_area;
            if !handled && col >= a.x && col < a.x + a.width && row >= a.y + 1 && row < a.y + a.height {
                let idx = (row - a.y - 1) as usize;
                if idx < app.tasks.len() {
                    let is_double = app.last_click.as_ref()
                        .map(|&(lc, lr, ref t)| lc == col && lr == row && t.elapsed().as_millis() < 400)
                        .unwrap_or(false);
                    app.list_state.select(Some(idx));
                    if is_double {
                        app.focus = Focus::Content;
                        app.spawn_active_pty();
                    } else {
                        app.focus = Focus::Sidebar;
                    }
                    handled = true;
                }
            }

            // Topbar tabs
            if !handled && in_rect(col, row, app.topbar_area) {
                app.focus = Focus::Topbar;
                // New tab button
                if in_rect(col, row, app.new_tab_area) {
                    app.cur_task_mut().new_tab();
                    app.spawn_active_pty();
                    app.dirty = true;
                    handled = true;
                }
                // Tab click
                if !handled {
                    let tab_areas = app.tab_areas.clone();
                    for (i, ta) in tab_areas.iter().enumerate() {
                        if in_rect(col, row, *ta) {
                            let ti = app.selected();
                            let is_double = app.last_click.as_ref()
                                .map(|&(lc, lr, ref t)| lc == col && lr == row && t.elapsed().as_millis() < 400)
                                .unwrap_or(false);
                            if is_double && app.tasks[ti].active_tab == i {
                                app.begin_rename_tab();
                            } else {
                                app.tasks[ti].active_tab = i;
                                app.spawn_active_pty();
                            }
                            handled = true;
                            break;
                        }
                    }
                }
            }

            // Content area
            if !handled && in_rect(col, row, app.content_area) {
                app.focus = Focus::Content; handled = true;
            }
            if !handled && in_rect(col, row, app.sidebar_area) {
                app.focus = Focus::Sidebar;
            }

            app.last_click = Some((col, row, std::time::Instant::now()));
        }
        MouseEventKind::ScrollUp => {
            if in_rect(col, row, app.sidebar_area) { app.nav(-1); }
            else if in_rect(col, row, app.topbar_area) {
                app.tab_scroll = app.tab_scroll.saturating_sub(1);
            }
        }
        MouseEventKind::ScrollDown => {
            if in_rect(col, row, app.sidebar_area) { app.nav(1); }
            else if in_rect(col, row, app.topbar_area) {
                let max = app.tasks[app.selected()].tabs.len().saturating_sub(1);
                if app.tab_scroll < max { app.tab_scroll += 1; }
            }
        }
        _ => {}
    }
}
