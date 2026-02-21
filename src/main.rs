mod client;
mod terminal_provider;
mod terminal_emulator;
mod protocol;
mod daemon_modules;
mod providers;
mod emulators;
mod config;
mod utils;
mod error;
mod theme;

use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use error::{MatoError, Result};

#[cfg(unix)]
extern crate libc;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use client::{App, save_state};
use client::ui::draw;
use client::input::handle_key;
use client::app::Focus;

// Global flag for SIGCONT
static RESUMED: AtomicBool = AtomicBool::new(false);

#[cfg(unix)]
extern "C" fn handle_sigcont(_: libc::c_int) {
    RESUMED.store(true, Ordering::Relaxed);
}

fn main() -> Result<()> {
    // Check for --version flag
    if std::env::args().any(|a| a == "--version" || a == "-v") {
        println!("mato {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Check for --daemon flag
    if std::env::args().any(|a| a == "--daemon") {
        let foreground = std::env::args().any(|a| a == "--foreground");
        return daemon_modules::run_daemon(foreground).map_err(MatoError::from);
    }

    // Check for --status flag
    if std::env::args().any(|a| a == "--status") {
        return daemon_modules::show_status().map_err(MatoError::from);
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
        client::show_onboarding_tui()?;
    }

    // Ensure daemon is running
    daemon_modules::ensure_daemon_running()?;

    run_client().map_err(|e| {
        eprintln!("Error: {}", e);
        e
    })
}

fn run_client() -> Result<()> {
    // Setup SIGCONT handler
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGCONT, handle_sigcont as libc::sighandler_t);
    }
    
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.show_cursor()?;  // Show cursor at startup

    let mut app = App::new();
    terminal.draw(|f| draw(f, &mut app))?;
    app.spawn_active_pty();

    loop {
        // Check if we resumed from suspend (SIGCONT)
        if RESUMED.swap(false, Ordering::Relaxed) {
            // Reinitialize terminal after resume
            enable_raw_mode()?;
            execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
            terminal.clear()?;
        }
        
        // Update active status and spinner animation
        app.refresh_active_status();
        app.refresh_update_status();
        app.update_spinner();
        
        terminal.draw(|f| draw(f, &mut app))?;
        
        // Apply pending resize after user stops resizing
        app.apply_pending_resize();
        
        // Poll at ~12fps for smooth animation (80ms spinner frame + some overhead)
        let timeout = if app.has_active_tabs() {
            Duration::from_millis(80)  // Fast polling when active
        } else {
            Duration::from_millis(200)  // Slower when idle
        };
        
        if event::poll(timeout)? {
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
