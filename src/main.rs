mod client;
mod terminal_provider;
mod terminal_emulator;
mod protocol;
mod daemon;
mod providers;
mod emulators;
mod config;
mod utils;
mod error;
mod theme;
mod terminal;

use std::time::Duration;
use error::{MatoError, Result};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use client::{App, save_state};
use client::ui::draw;
use client::input::handle_key;
use client::app::{Focus, Office, Desk, TabEntry};
use terminal::{consume_resumed, TerminalGuard};

fn main() -> Result<()> {
    // Check for --version flag
    if std::env::args().any(|a| a == "--version" || a == "-v") {
        println!("mato {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Check for --daemon flag
    if std::env::args().any(|a| a == "--daemon") {
        let foreground = std::env::args().any(|a| a == "--foreground");
        return daemon::run_daemon(foreground).map_err(MatoError::from);
    }

    // Check for --status flag
    if std::env::args().any(|a| a == "--status") {
        return daemon::show_status().map_err(MatoError::from);
    }

    // Check for --kill flag
    if std::env::args().any(|a| a == "--kill") {
        return daemon::kill_all().map_err(MatoError::from);
    }

    // Setup client logging
    let log_path = utils::get_client_log_path();
    if let Ok(log_file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = tracing_subscriber::fmt()
            .with_writer(log_file)
            .with_ansi(false)
            .try_init();
        tracing::info!("=== Client starting ===");
    }

    // Check if this is first run (no state file)
    let state_path = utils::get_state_file_path();
    if !state_path.exists() {
        if let Some(state) = client::show_onboarding_tui()? {
            let state_json = serde_json::to_string_pretty(&state)
                .map_err(|e| MatoError::Io(std::io::Error::other(e)))?;
            if let Some(parent) = state_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&state_path, state_json)?;
        }
    }

    // Ensure daemon is running
    daemon::ensure_daemon_running()?;

    run_client().map_err(|e| {
        eprintln!("Error: {}", e);
        e
    })
}

fn run_client() -> Result<()> {
    let _terminal_guard = TerminalGuard::new();
    
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, crossterm::event::EnableBracketedPaste)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.show_cursor()?;  // Show cursor at startup

    let mut app = App::new();
    terminal.draw(|f| draw(f, &mut app))?;
    app.spawn_active_pty();

    loop {
        // Check if we resumed from suspend (SIGCONT)
        if consume_resumed() {
            // Reinitialize terminal after resume
            enable_raw_mode()?;
            execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture, crossterm::event::EnableBracketedPaste)?;
            terminal.clear()?;
            app.last_cursor_shape = None;
        }
        
        // Update active status and spinner animation
        app.refresh_active_status();
        app.refresh_update_status();
        app.update_spinner();
        app.sync_tab_titles();
        app.sync_focus_events();
        
        terminal.draw(|f| draw(f, &mut app))?;
        if let Some(elapsed) = app.finish_tab_switch_measurement() {
            tracing::info!("Tab switch first-frame latency: {}ms", elapsed.as_millis());
        }
        
        // Apply pending resize after user stops resizing
        app.apply_pending_resize();
        
        // Poll at ~12fps for smooth animation (80ms spinner frame + some overhead)
        let timeout = if app.has_active_tabs() || matches!(app.focus, client::app::Focus::Content) {
            Duration::from_millis(80)  // Fast polling when active
        } else {
            Duration::from_millis(200)  // Slower when idle
        };
        
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    if handle_key(&mut app, key) {
                        break;
                    }
                    if app.dirty {
                        if let Err(e) = save_state(&app) {
                            tracing::warn!("Failed to save state: {}", e);
                        }
                        app.dirty = false;
                    }
                }
                Event::Mouse(me) => handle_mouse(&mut app, me),
                Event::Resize(_, _) => {
                    // Terminal resized - trigger PTY resize via pending mechanism
                    app.resize_all_ptys(app.term_rows, app.term_cols);
                }
                Event::Paste(text) => {
                    if matches!(app.focus, client::app::Focus::Content) {
                        app.pty_paste(&text);
                    }
                }
                _ => {}
            }
        }
        
        if app.should_show_onboarding {
            app.should_show_onboarding = false;
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture, crossterm::event::DisableBracketedPaste)?;
            
            if let Some(state) = client::show_onboarding_tui()? {
                let new_office_idx = app.offices.len();
                let new_office = state.offices.into_iter().next().unwrap();
                let office = Office {
                    id: new_office.id,
                    name: new_office.name,
                    desks: new_office.desks.into_iter().map(|d| {
                        let tabs = d.tabs.into_iter().map(|tb| TabEntry::with_id(tb.id, tb.name)).collect();
                        Desk { id: d.id, name: d.name, tabs, active_tab: d.active_tab }
                    }).collect(),
                    active_desk: new_office.active_desk,
                };
                app.offices.push(office);
                app.switch_office(new_office_idx);
                if let Err(e) = save_state(&app) {
                    tracing::warn!("Failed to save state after onboarding: {}", e);
                }
            }
            
            enable_raw_mode()?;
            execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture, crossterm::event::EnableBracketedPaste)?;
            terminal.clear()?;
            app.last_cursor_shape = None;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture, crossterm::event::DisableBracketedPaste)?;
    terminal.show_cursor()?;
    Ok(())
}

fn handle_mouse(app: &mut App, me: crossterm::event::MouseEvent) {
    let (col, row) = (me.column, me.row);
    fn in_rect(col: u16, row: u16, r: Rect) -> bool {
        col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height
    }

    // Mouse passthrough to PTY when in content focus
    if matches!(app.focus, client::app::Focus::Content) && in_rect(col, row, app.content_area) {
        let tx = col.saturating_sub(app.content_area.x + 1) + 1;
        let ty = row.saturating_sub(app.content_area.y + 1) + 1;
        let mouse_mode = app.pty_mouse_mode_enabled();
        match me.kind {
            MouseEventKind::ScrollUp => {
                if mouse_mode {
                    app.pty_write(format!("\x1b[<64;{};{}M", tx, ty).as_bytes());
                } else {
                    app.pty_scroll(3);
                }
                return;
            }
            MouseEventKind::ScrollDown => {
                if mouse_mode {
                    app.pty_write(format!("\x1b[<65;{};{}M", tx, ty).as_bytes());
                } else {
                    app.pty_scroll(-3);
                }
                return;
            }
            _ => {}
        }
        if !mouse_mode {
            return;
        }
        let (btn, is_up) = match me.kind {
            MouseEventKind::Down(MouseButton::Left)   => (0u8, false),
            MouseEventKind::Down(MouseButton::Middle) => (1,   false),
            MouseEventKind::Down(MouseButton::Right)  => (2,   false),
            MouseEventKind::Up(_)                     => (3,   true),
            MouseEventKind::Drag(MouseButton::Left)   => (32,  false),
            MouseEventKind::Moved                     => (35,  false),
            _ => return,
        };
        let suffix = if is_up { 'm' } else { 'M' };
        app.pty_write(format!("\x1b[<{};{};{}{}", btn, tx, ty, suffix).as_bytes());
        return;
    }

    match me.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let mut handled = false;

            // Office selector
            if in_rect(col, row, app.new_desk_area) {
                app.office_selector.active = true;
                app.office_selector.list_state.select(Some(app.current_office));
                handled = true;
            }

            // Sidebar desk list
            let a = app.sidebar_list_area;
            if !handled && col >= a.x && col < a.x + a.width && row > a.y && row < a.y + a.height {
                let idx = (row - a.y - 1) as usize;
                if idx < app.offices[app.current_office].desks.len() {
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
                    app.cur_desk_mut().new_tab();
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
                            if is_double && app.offices[app.current_office].desks[ti].active_tab == i {
                                app.begin_rename_tab();
                            } else {
                                app.offices[app.current_office].desks[ti].active_tab = i;
                                app.mark_tab_switch();
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
            } else if in_rect(col, row, app.content_area) {
                app.pty_scroll(3);
            }
        }
        MouseEventKind::ScrollDown => {
            if in_rect(col, row, app.sidebar_area) { app.nav(1); }
            else if in_rect(col, row, app.topbar_area) {
                let max = app.offices[app.current_office].desks[app.selected()].tabs.len().saturating_sub(1);
                if app.tab_scroll < max { app.tab_scroll += 1; }
            } else if in_rect(col, row, app.content_area) {
                app.pty_scroll(-3);
            }
        }
        _ => {}
    }
}
