use crate::client::app::{
    App, Focus, JumpMode, OfficeDeleteConfirm, RenameState, RenameTarget,
    CONTENT_ESC_DOUBLE_PRESS_WINDOW_MS,
};
use crossterm::{
    event::{EnableMouseCapture, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use std::io;
use std::time::{Duration, Instant};

fn hard_disable_terminal_modes() {
    crate::terminal::restore_terminal_modes();
}

pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    if key.code == KeyCode::Esc && key.kind == KeyEventKind::Repeat {
        return false;
    }
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

    // Emergency exit when daemon connection is unhealthy.
    // This provides a reliable way out when the top-right status is blinking/disconnected.
    if !app.daemon_connected
        && (matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q'))
            || (ctrl && key.code == KeyCode::Char('c')))
    {
        return true;
    }

    if app.copy_mode {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => app.copy_mode = false,
            KeyCode::Up | KeyCode::Char('k') => app.pty_scroll(3),
            KeyCode::Down | KeyCode::Char('j') => app.pty_scroll(-3),
            KeyCode::PageUp => app.pty_scroll(20),
            KeyCode::PageDown => app.pty_scroll(-20),
            KeyCode::Char('g') => app.pty_scroll(1_000_000),
            KeyCode::Char('G') => app.pty_scroll(-1_000_000),
            _ => {}
        }
        return false;
    }

    // Office delete confirmation intercepts keys
    if let Some(confirm) = app.desk_delete_confirm.as_ref() {
        let desk_idx = confirm.desk_idx;
        match key.code {
            KeyCode::Char('y' | 'Y') | KeyCode::Enter => {
                app.confirm_close_desk(desk_idx);
            }
            KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                app.desk_delete_confirm = None;
            }
            _ => {}
        }
        return false;
    }

    // Office delete confirmation intercepts keys
    if let Some(ref mut confirm) = app.office_delete_confirm {
        match key.code {
            KeyCode::Esc => {
                app.office_delete_confirm = None;
            }
            KeyCode::Char(c) => {
                confirm.input.push(c);
            }
            KeyCode::Backspace => {
                confirm.input.pop();
            }
            KeyCode::Enter => {
                let office_idx = confirm.office_idx;
                let expected_name = &app.offices[office_idx].name;
                if confirm.input == *expected_name && app.offices.len() > 1 {
                    app.offices.remove(office_idx);
                    if app.current_office >= app.offices.len() {
                        app.current_office = app.offices.len() - 1;
                    }
                    app.switch_office(app.current_office);
                    app.dirty = true;
                }
                app.office_delete_confirm = None;
            }
            _ => {}
        }
        return false;
    }

    // Office selector intercepts keys
    if app.office_selector.active {
        let max_idx = app.offices.len();
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.office_selector.active = false;
            }
            KeyCode::Up => {
                let selected = app.office_selector.list_state.selected().unwrap_or(0);
                if selected > 0 {
                    app.office_selector.list_state.select(Some(selected - 1));
                }
            }
            KeyCode::Down => {
                let selected = app.office_selector.list_state.selected().unwrap_or(0);
                if selected < max_idx {
                    app.office_selector.list_state.select(Some(selected + 1));
                }
            }
            KeyCode::Char('r') => {
                let selected = app.office_selector.list_state.selected().unwrap_or(0);
                if selected < app.offices.len() {
                    let office_name = app.offices[selected].name.clone();
                    app.rename = Some(RenameState::new(
                        RenameTarget::Office(selected),
                        office_name,
                    ));
                    app.office_selector.active = false;
                }
            }
            KeyCode::Char('d') => {
                let selected = app.office_selector.list_state.selected().unwrap_or(0);
                if selected < app.offices.len() && app.offices.len() > 1 {
                    app.office_delete_confirm = Some(OfficeDeleteConfirm::new(selected));
                    app.office_selector.active = false;
                }
            }
            KeyCode::Enter => {
                let selected = app.office_selector.list_state.selected().unwrap_or(0);
                if selected < app.offices.len() {
                    app.switch_office(selected);
                    app.office_selector.active = false;
                } else {
                    app.office_selector.active = false;
                    app.should_show_onboarding = true;
                }
            }
            _ => {}
        }
        return false;
    }

    // Settings screen intercepts keys
    if app.show_settings {
        match key.code {
            KeyCode::Esc | KeyCode::Char('s') | KeyCode::Char('q') => {
                app.show_settings = false;
            }
            KeyCode::Up => {
                if app.settings_selected > 0 {
                    app.settings_selected -= 1;
                }
            }
            KeyCode::Down => {
                if app.settings_selected + 1 < crate::theme::BUILTIN_THEMES.len() {
                    app.settings_selected += 1;
                }
            }
            KeyCode::Enter => {
                let name = crate::theme::BUILTIN_THEMES[app.settings_selected];
                app.theme = crate::theme::builtin(name);
                crate::theme::save_name(name).ok();
                app.show_settings = false;
            }
            _ => {}
        }
        return false;
    }

    // Jump mode intercepts all keys
    if let JumpMode::Active = app.jump_mode {
        match key.code {
            KeyCode::Char('q' | 'Q') => return true, // Quit in Jump Mode
            KeyCode::Char('c') if app.focus == Focus::Content => {
                app.copy_mode = true;
                app.jump_mode = JumpMode::None;
            }
            KeyCode::Char(c) if matches!(c, 'r' | 'R') && app.focus == Focus::Sidebar => {
                let i = app.selected();
                app.begin_rename_desk(i);
                app.jump_mode = JumpMode::None;
            }
            KeyCode::Char(c) if matches!(c, 'r' | 'R') && app.focus == Focus::Topbar => {
                app.begin_rename_tab();
                app.jump_mode = JumpMode::None;
            }
            KeyCode::Char(c) if matches!(c, 'r' | 'R') && app.focus == Focus::Content => {
                app.restart_active_pty();
                app.jump_mode = JumpMode::None;
            }
            KeyCode::Esc => {
                app.jump_mode = JumpMode::None;
            }
            // Focus switching matrix in Jump Mode:
            // Topbar:  ← Sidebar, ↓ Content
            // Sidebar: ↑ Topbar, → Content
            // Content: ← Sidebar, ↑ Topbar
            KeyCode::Left if matches!(app.focus, Focus::Topbar | Focus::Content) => {
                app.focus = Focus::Sidebar;
                app.jump_mode = JumpMode::None;
            }
            KeyCode::Down if app.focus == Focus::Topbar => {
                app.focus = Focus::Content;
                app.jump_mode = JumpMode::None;
            }
            KeyCode::Right if app.focus == Focus::Sidebar => {
                app.focus = Focus::Content;
                app.jump_mode = JumpMode::None;
            }
            KeyCode::Up if matches!(app.focus, Focus::Sidebar | Focus::Content) => {
                app.focus = Focus::Topbar;
                app.jump_mode = JumpMode::None;
            }
            // Alphanumeric keys for jumping (filtered by focus-specific reserved keys).
            KeyCode::Char(c) if c.is_ascii_alphanumeric() => {
                app.handle_jump_selection(c);
            }
            _ => {}
        }
        return false;
    }

    // Rename mode intercepts all keys
    if app.rename.is_some() {
        match key.code {
            KeyCode::Enter => app.commit_rename(),
            KeyCode::Esc => app.cancel_rename(),
            KeyCode::Backspace => {
                if let Some(rename) = &mut app.rename {
                    rename.backspace();
                }
            }
            KeyCode::Delete => {
                if let Some(rename) = &mut app.rename {
                    rename.delete();
                }
            }
            KeyCode::Left => {
                if let Some(rename) = &mut app.rename {
                    rename.move_left();
                }
            }
            KeyCode::Right => {
                if let Some(rename) = &mut app.rename {
                    rename.move_right();
                }
            }
            KeyCode::Home => {
                if let Some(rename) = &mut app.rename {
                    rename.move_home();
                }
            }
            KeyCode::End => {
                if let Some(rename) = &mut app.rename {
                    rename.move_end();
                }
            }
            KeyCode::Char(c) if !ctrl => {
                if let Some(rename) = &mut app.rename {
                    rename.insert_char(c);
                }
            }
            _ => {}
        }
        return false;
    }

    // Ctrl+Z suspend
    if ctrl && key.code == KeyCode::Char('z') && app.focus != Focus::Content {
        #[cfg(unix)]
        {
            use std::io::Write;
            hard_disable_terminal_modes();

            // Send SIGTSTP to self
            unsafe {
                libc::kill(libc::getpid(), libc::SIGTSTP);
            }

            // After resume (fg) - reinitialize everything
            enable_raw_mode().ok();
            execute!(
                io::stdout(),
                EnterAlternateScreen,
                EnableMouseCapture,
                crossterm::event::EnableBracketedPaste
            )
            .ok();
            io::stdout().flush().ok();
        }
        return false;
    }

    // In Content focus, immediately forward any pending single-ESC before
    // handling a non-ESC key. ESC+key combinations (Alt sequences in readline,
    // ESC before a vim command, etc.) must arrive at the PTY in order.
    // Note: the timer-based flush (main.rs) uses flush_pending_content_esc()
    // which has a 300ms guard; here we always forward immediately.
    if app.focus == Focus::Content && !matches!(key.code, KeyCode::Esc) {
        if app.last_content_esc.take().is_some() {
            app.pty_write(b"\x1b");
        }
    }

    if key.code == KeyCode::Esc {
        if app.focus == Focus::Content {
            // In Content focus: double-ESC enters Jump Mode.
            // Single ESC is delayed briefly, then forwarded to shell.
            let now = Instant::now();
            if let Some(prev) = app.last_content_esc {
                if now.duration_since(prev)
                    < Duration::from_millis(CONTENT_ESC_DOUBLE_PRESS_WINDOW_MS)
                {
                    app.last_content_esc = None;
                    app.jump_mode = JumpMode::Active;
                    return false;
                }
                // Previous ESC is no longer a double-press candidate; forward it now.
                app.last_content_esc = None;
                app.pty_write(b"\x1b");
            }
            app.last_content_esc = Some(now);
            return false;
        }
        // Non-Content focus: single ESC enters Jump Mode
        app.jump_mode = JumpMode::Active;
        return false;
    }

    // Alt+1-9: switch tab
    if alt {
        if let KeyCode::Char(c) = key.code {
            if let Some(n) = c.to_digit(10) {
                let idx = (n as usize).saturating_sub(1);
                let i = app.selected();
                if idx < app.offices[app.current_office].desks[i].tabs.len() {
                    app.pty_send_focus_event(false);
                    app.offices[app.current_office].desks[i].active_tab = idx;
                    app.mark_tab_switch();
                    app.spawn_active_pty();
                    app.pty_send_focus_event(true);
                }
                return false;
            }
        }
    }

    match app.focus {
        Focus::Sidebar => match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('o') => {
                app.office_selector.active = true;
                app.office_selector
                    .list_state
                    .select(Some(app.current_office));
            }
            KeyCode::Char('n') => app.new_desk(),
            KeyCode::Char('x') => app.request_close_desk(),
            KeyCode::Char('r' | 'R') => {
                let i = app.selected();
                app.begin_rename_desk(i);
            }
            KeyCode::Char('s') => {
                app.show_settings = true;
            }
            KeyCode::Up => app.nav(-1),
            KeyCode::Down => app.nav(1),
            KeyCode::Enter => {
                app.focus = Focus::Content;
                app.spawn_active_pty();
            }
            _ => {}
        },
        Focus::Topbar => match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Left => {
                let i = app.selected();
                let at = app.offices[app.current_office].desks[i].active_tab;
                if at > 0 {
                    app.offices[app.current_office].desks[i].active_tab = at - 1;
                    app.mark_tab_switch();
                    app.spawn_active_pty();
                }
            }
            KeyCode::Right => {
                let i = app.selected();
                let len = app.offices[app.current_office].desks[i].tabs.len();
                let at = app.offices[app.current_office].desks[i].active_tab;
                if at + 1 < len {
                    app.offices[app.current_office].desks[i].active_tab = at + 1;
                    app.mark_tab_switch();
                    app.spawn_active_pty();
                }
            }
            KeyCode::Char('n') => {
                app.new_tab_inheriting_cwd();
                app.spawn_active_pty();
                app.dirty = true;
            }
            KeyCode::Char('x') => {
                app.cur_desk_mut().close_tab();
                app.mark_tab_switch();
                app.spawn_active_pty();
                app.dirty = true;
            }
            KeyCode::Char('r' | 'R') => app.begin_rename_tab(),
            KeyCode::Enter => {
                app.focus = Focus::Content;
                app.spawn_active_pty();
            }
            _ => {}
        },
        Focus::Content => {
            let shift = key
                .modifiers
                .contains(crossterm::event::KeyModifiers::SHIFT);
            // Shift+PageUp/Down = scrollback
            if shift && key.code == KeyCode::PageUp {
                app.pty_scroll(5);
                return false;
            }
            if shift && key.code == KeyCode::PageDown {
                app.pty_scroll(-5);
                return false;
            }
            let bytes = encode_content_key(&key);
            if !bytes.is_empty() {
                app.pty_write(&bytes);
            }
        }
    }
    false
}

fn encode_content_key(key: &crossterm::event::KeyEvent) -> Vec<u8> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let mut bytes: Vec<u8> = match key.code {
        KeyCode::Enter => b"\r".to_vec(),
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Tab => b"\t".to_vec(),
        KeyCode::BackTab => b"\x1b[Z".to_vec(),
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        KeyCode::Insert => b"\x1b[2~".to_vec(),
        KeyCode::PageUp => b"\x1b[5~".to_vec(),
        KeyCode::PageDown => b"\x1b[6~".to_vec(),
        KeyCode::F(n) => encode_function_key(n),
        KeyCode::Char(c) => {
            if ctrl {
                encode_ctrl_char(c)
            } else {
                c.to_string().into_bytes()
            }
        }
        _ => vec![],
    };

    if alt && !bytes.is_empty() {
        let mut prefixed = Vec::with_capacity(bytes.len() + 1);
        prefixed.push(0x1b);
        prefixed.extend_from_slice(&bytes);
        bytes = prefixed;
    }

    bytes
}

fn encode_ctrl_char(c: char) -> Vec<u8> {
    match c {
        'a'..='z' => vec![(c as u8 - b'a') + 1],
        'A'..='Z' => vec![(c as u8 - b'A') + 1],
        ' ' | '@' => vec![0x00],
        '[' => vec![0x1b],
        '\\' => vec![0x1c],
        ']' => vec![0x1d],
        '^' => vec![0x1e],
        '_' => vec![0x1f],
        '?' => vec![0x7f],
        _ => vec![],
    }
}

fn encode_function_key(n: u8) -> Vec<u8> {
    match n {
        1 => b"\x1bOP".to_vec(),
        2 => b"\x1bOQ".to_vec(),
        3 => b"\x1bOR".to_vec(),
        4 => b"\x1bOS".to_vec(),
        5 => b"\x1b[15~".to_vec(),
        6 => b"\x1b[17~".to_vec(),
        7 => b"\x1b[18~".to_vec(),
        8 => b"\x1b[19~".to_vec(),
        9 => b"\x1b[20~".to_vec(),
        10 => b"\x1b[21~".to_vec(),
        11 => b"\x1b[23~".to_vec(),
        12 => b"\x1b[24~".to_vec(),
        _ => vec![],
    }
}
