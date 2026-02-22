mod client;
mod config;
mod daemon;
mod emulators;
mod error;
mod protocol;
mod providers;
mod terminal;
mod terminal_emulator;
mod terminal_provider;
mod theme;
mod utils;

use error::{MatoError, Result};
use protocol::{ClientMsg, ServerMsg};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use crossterm::{
    cursor::MoveTo,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, MouseButton, MouseEventKind},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use client::app::{Desk, Focus, Office, TabEntry};
use client::input::handle_key;
use client::ui::draw;
use client::{save_state, App, OnboardingAction, OnboardingController};
use terminal::{consume_resumed, TerminalGuard};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut want_help = false;
    let mut want_version = false;
    let mut want_daemon = false;
    let mut want_foreground = false;
    let mut want_status = false;
    let mut want_kill = false;
    let mut unknown: Vec<String> = Vec::new();

    for arg in &args {
        match arg.as_str() {
            "--help" | "-h" | "help" => want_help = true,
            "--version" | "-v" => want_version = true,
            "--daemon" => want_daemon = true,
            "--foreground" => want_foreground = true,
            "--status" => want_status = true,
            "--kill" => want_kill = true,
            _ => unknown.push(arg.clone()),
        }
    }

    if want_help {
        print_help();
        return Ok(());
    }

    if !unknown.is_empty() {
        eprintln!("Unknown argument(s): {}", unknown.join(" "));
        eprintln!();
        print_help();
        std::process::exit(2);
    }

    let mode_count =
        (want_version as u8) + (want_daemon as u8) + (want_status as u8) + (want_kill as u8);
    if mode_count > 1 {
        eprintln!(
            "Conflicting command flags. Use only one of: --version, --daemon, --status, --kill"
        );
        eprintln!();
        print_help();
        std::process::exit(2);
    }

    if want_foreground && !want_daemon {
        eprintln!("--foreground can only be used with --daemon");
        eprintln!();
        print_help();
        std::process::exit(2);
    }

    if want_version {
        println!("mato {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if want_daemon {
        return daemon::run_daemon(want_foreground).map_err(MatoError::from);
    }

    if want_status {
        return daemon::show_status().map_err(MatoError::from);
    }

    if want_kill {
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
        } else {
            return Ok(());
        }
    }

    // Ensure daemon is running
    daemon::ensure_daemon_running()?;
    ensure_daemon_version_compatible()?;

    run_client().map_err(|e| {
        eprintln!("Error: {}", e);
        e
    })
}

fn daemon_version() -> Option<String> {
    let socket_path = utils::get_socket_path();
    let mut stream = UnixStream::connect(&socket_path).ok()?;
    let hello = ClientMsg::Hello {
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let json = serde_json::to_vec(&hello).ok()?;
    stream.write_all(&json).ok()?;
    stream.write_all(b"\n").ok()?;
    stream.flush().ok()?;

    let mut line = String::new();
    BufReader::new(&stream).read_line(&mut line).ok()?;
    match serde_json::from_str::<ServerMsg>(&line).ok()? {
        ServerMsg::Welcome { version } => Some(version),
        _ => None,
    }
}

fn confirm_daemon_restart(client_version: &str, daemon_version: &str) -> bool {
    eprintln!(
        "Daemon version mismatch: daemon={}, client={}.",
        daemon_version, client_version
    );
    eprintln!("Restarting daemon will:");
    eprintln!("- terminate all running TTY/shell processes");
    eprintln!("- close other running mato clients");
    eprintln!("- keep layout/state from saved config, but lose live process state");
    eprint!("Restart daemon now to use the new version? [y/N]: ");
    let _ = std::io::stderr().flush();
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        return false;
    }
    let answer = input.trim().to_ascii_lowercase();
    answer == "y" || answer == "yes"
}

fn ensure_daemon_version_compatible() -> Result<()> {
    let client_version = env!("CARGO_PKG_VERSION");
    let Some(daemon_ver) = daemon_version() else {
        return Ok(());
    };

    if daemon_ver == client_version {
        return Ok(());
    }

    if confirm_daemon_restart(client_version, &daemon_ver) {
        daemon::kill_all()?;
        daemon::ensure_daemon_running()?;
        if let Some(v) = daemon_version() {
            if v != client_version {
                eprintln!(
                    "Warning: daemon restarted but version is still {} (expected {}).",
                    v, client_version
                );
            }
        }
    } else {
        eprintln!("Continuing with existing daemon version {}.", daemon_ver);
    }

    Ok(())
}

fn print_help() {
    println!(
        "mato {}\n\
         Multi-Agent Terminal Office\n\n\
         Usage:\n\
           mato                    Start client UI (auto-start daemon if needed)\n\
           mato --daemon           Run daemon in background mode\n\
           mato --daemon --foreground\n\
                                   Run daemon in foreground (debug)\n\
           mato --status           Show daemon/runtime status\n\
           mato --kill             Kill daemon, clients, and managed tab processes\n\
           mato --version, -v      Show version\n\
           mato --help, -h, help   Show this help\n",
        env!("CARGO_PKG_VERSION")
    );
}

enum ScreenState {
    Main,
    Onboarding(OnboardingController),
}

fn apply_onboarding_state(app: &mut App, state: client::persistence::SavedState) {
    let new_office_idx = app.offices.len();
    let new_office = state.offices.into_iter().next().unwrap();
    let office = Office {
        id: new_office.id,
        name: new_office.name,
        desks: new_office
            .desks
            .into_iter()
            .map(|d| {
                let tabs = d
                    .tabs
                    .into_iter()
                    .map(|tb| TabEntry::with_id(tb.id, tb.name))
                    .collect();
                Desk {
                    id: d.id,
                    name: d.name,
                    tabs,
                    active_tab: d.active_tab,
                }
            })
            .collect(),
        active_desk: new_office.active_desk,
    };
    app.offices.push(office);
    app.switch_office(new_office_idx);
}

fn run_client() -> Result<()> {
    let _terminal_guard = TerminalGuard::new();

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        crossterm::event::EnableBracketedPaste
    )?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.hide_cursor()?;
    let mut mouse_capture_enabled = true;

    let mut app = App::new();
    app.spawn_active_pty();
    terminal.draw(|f| draw(f, &mut app))?;
    let mut screen = ScreenState::Main;
    let mut last_input_at = Instant::now() - Duration::from_secs(10);

    loop {
        // Check if we resumed from suspend (SIGCONT)
        if consume_resumed() {
            // Reinitialize terminal after resume
            enable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                EnterAlternateScreen,
                EnableMouseCapture,
                crossterm::event::EnableBracketedPaste
            )?;
            terminal.clear()?;
            mouse_capture_enabled = true;
        }
        match &mut screen {
            ScreenState::Main => {
                // Keep mouse capture enabled globally so Topbar/Sidebar remain clickable
                // even while Content is focused. Exception: Copy Mode, where we
                // deliberately let the host terminal own mouse selection/copy.
                let desired_mouse_capture = !app.copy_mode;
                if desired_mouse_capture != mouse_capture_enabled {
                    if desired_mouse_capture {
                        execute!(terminal.backend_mut(), EnableMouseCapture)?;
                    } else {
                        execute!(terminal.backend_mut(), DisableMouseCapture)?;
                    }
                    mouse_capture_enabled = desired_mouse_capture;
                }

                app.refresh_active_status();
                app.refresh_update_status();
                app.sync_tab_titles();
                app.sync_focus_events();

                // Skip render if screen content hasn't changed (push mode dedup)
                let current_gen = app.active_provider_screen_generation();
                let ui_changed = app.dirty
                    || app.pending_bell
                    || (!app.copy_mode && current_gen != app.last_rendered_screen_gen);

                if ui_changed || last_input_at.elapsed() < Duration::from_millis(100) {
                    app.update_spinner();
                    terminal.draw(|f| draw(f, &mut app))?;
                    app.last_rendered_screen_gen = current_gen;
                }

                // Forward bell (BEL) from inner terminal to host terminal.
                if app.pending_bell {
                    app.pending_bell = false;
                    let _ = execute!(terminal.backend_mut(), crossterm::style::Print("\x07"));
                }

                if let Some(elapsed) = app.finish_tab_switch_measurement() {
                    tracing::debug!("Tab switch first-frame latency: {}ms", elapsed.as_millis());
                }

                // Apply pending resize after user stops resizing
                app.apply_pending_resize();
            }
            ScreenState::Onboarding(controller) => {
                terminal.draw(|f| controller.draw(f))?;
            }
        }

        // Adaptive poll: very short after recent input for fast echo,
        // normal rate otherwise. With push mode, screen updates arrive
        // asynchronously and bump screen_generation — we only render when needed.
        let mut timeout = match &screen {
            ScreenState::Main => {
                let since_input = last_input_at.elapsed();
                if since_input < Duration::from_millis(50) {
                    Duration::from_millis(1) // Ultra-fast echo after input
                } else if since_input < Duration::from_millis(200) {
                    Duration::from_millis(8) // Quick follow-up for command output
                } else if app.has_active_tabs() || matches!(app.focus, client::app::Focus::Content)
                {
                    Duration::from_millis(16) // ~60fps when active
                } else {
                    Duration::from_millis(100) // Idle
                }
            }
            ScreenState::Onboarding(_) => Duration::from_millis(200),
        };

        // Drain ALL pending events before rendering to avoid wasting frames.
        let mut should_break = false;
        let mut had_content_input = false;
        while event::poll(timeout)? {
            // After first event, use zero timeout to drain remaining
            timeout = Duration::ZERO;
            match event::read()? {
                Event::Key(key) => match &mut screen {
                    ScreenState::Main => {
                        // Any key in Main should trigger near-immediate UI redraw
                        // (e.g. opening rename popup from Sidebar/Topbar).
                        last_input_at = Instant::now();
                        if matches!(app.focus, client::app::Focus::Content) {
                            had_content_input = true;
                        }
                        if handle_key(&mut app, key) {
                            should_break = true;
                            break;
                        }
                        if app.dirty {
                            if let Err(e) = save_state(&app) {
                                tracing::warn!("Failed to save state: {}", e);
                            }
                            app.dirty = false;
                        }
                        if app.should_show_onboarding {
                            app.should_show_onboarding = false;
                            screen = ScreenState::Onboarding(OnboardingController::new_in_app());
                            terminal.clear()?;
                        }
                    }
                    ScreenState::Onboarding(controller) => match controller.handle_key(key) {
                        OnboardingAction::None => {}
                        OnboardingAction::Cancel => {
                            screen = ScreenState::Main;
                            terminal.clear()?;
                        }
                        OnboardingAction::Complete(state) => {
                            apply_onboarding_state(&mut app, state);
                            if let Err(e) = save_state(&app) {
                                tracing::warn!("Failed to save state after onboarding: {}", e);
                            }
                            screen = ScreenState::Main;
                            terminal.clear()?;
                        }
                    },
                },
                Event::Mouse(me) => {
                    if matches!(screen, ScreenState::Main) {
                        handle_mouse(&mut app, me);
                    }
                }
                Event::Resize(_, _) => {
                    // Terminal resized - trigger PTY resize via pending mechanism
                    if matches!(screen, ScreenState::Main) {
                        app.resize_all_ptys(app.term_rows, app.term_cols);
                    }
                }
                Event::Paste(text) => {
                    if matches!(screen, ScreenState::Main)
                        && matches!(app.focus, client::app::Focus::Content)
                    {
                        last_input_at = Instant::now();
                        app.pty_paste(&text);
                    }
                }
                _ => {}
            }
        }

        // Echo spin: after content input, briefly wait for echo to arrive
        // so we can render it in the same frame — eliminates one poll cycle (~2ms).
        if had_content_input && matches!(screen, ScreenState::Main) {
            let pre_gen = app.active_provider_screen_generation();
            let spin_deadline = Instant::now() + Duration::from_millis(3);
            while Instant::now() < spin_deadline {
                let new_gen = app.active_provider_screen_generation();
                if new_gen != pre_gen {
                    break; // Echo arrived!
                }
                std::thread::yield_now();
            }
        }

        if should_break {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        Clear(ClearType::All),
        MoveTo(0, 0),
        LeaveAlternateScreen,
        DisableMouseCapture,
        crossterm::event::DisableBracketedPaste
    )?;
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
            MouseEventKind::Down(MouseButton::Left) => (0u8, false),
            MouseEventKind::Down(MouseButton::Middle) => (1, false),
            MouseEventKind::Down(MouseButton::Right) => (2, false),
            MouseEventKind::Up(_) => (3, true),
            MouseEventKind::Drag(MouseButton::Left) => (32, false),
            MouseEventKind::Moved => (35, false),
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
                app.office_selector
                    .list_state
                    .select(Some(app.current_office));
                handled = true;
            }

            // Sidebar desk list
            let a = app.sidebar_list_area;
            if !handled && col >= a.x && col < a.x + a.width && row > a.y && row < a.y + a.height {
                let idx = (row - a.y - 1) as usize;
                if idx < app.offices[app.current_office].desks.len() {
                    let is_double = app
                        .last_click
                        .as_ref()
                        .map(|&(lc, lr, ref t)| {
                            lc == col && lr == row && t.elapsed().as_millis() < 400
                        })
                        .unwrap_or(false);
                    app.select_desk(idx);
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
                            let is_double = app
                                .last_click
                                .as_ref()
                                .map(|&(lc, lr, ref t)| {
                                    lc == col && lr == row && t.elapsed().as_millis() < 400
                                })
                                .unwrap_or(false);
                            if is_double
                                && app.offices[app.current_office].desks[ti].active_tab == i
                            {
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
                app.focus = Focus::Content;
                handled = true;
            }
            if !handled && in_rect(col, row, app.sidebar_area) {
                app.focus = Focus::Sidebar;
            }

            app.last_click = Some((col, row, std::time::Instant::now()));
        }
        MouseEventKind::ScrollUp => {
            if in_rect(col, row, app.sidebar_area) {
                app.nav(-1);
            } else if in_rect(col, row, app.topbar_area) {
                app.tab_scroll = app.tab_scroll.saturating_sub(1);
            } else if in_rect(col, row, app.content_area) {
                app.pty_scroll(3);
            }
        }
        MouseEventKind::ScrollDown => {
            if in_rect(col, row, app.sidebar_area) {
                app.nav(1);
            } else if in_rect(col, row, app.topbar_area) {
                let max = app.offices[app.current_office].desks[app.selected()]
                    .tabs
                    .len()
                    .saturating_sub(1);
                if app.tab_scroll < max {
                    app.tab_scroll += 1;
                }
            } else if in_rect(col, row, app.content_area) {
                app.pty_scroll(-3);
            }
        }
        _ => {}
    }
}
