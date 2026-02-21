use std::io;
use crossterm::{
    event::{KeyCode, KeyEventKind, KeyModifiers, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crate::client::app::{App, Focus, JumpMode, RenameTarget, OfficeDeleteConfirm};

pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    if key.code == KeyCode::Esc && key.kind == KeyEventKind::Repeat { return false; }
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

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
                if confirm.input == *expected_name {
                    if app.offices.len() > 1 {
                        app.offices.remove(office_idx);
                        if app.current_office >= app.offices.len() {
                            app.current_office = app.offices.len() - 1;
                        }
                        app.switch_office(app.current_office);
                        app.dirty = true;
                    }
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
                if selected + 1 <= max_idx {
                    app.office_selector.list_state.select(Some(selected + 1));
                }
            }
            KeyCode::Char('r') => {
                let selected = app.office_selector.list_state.selected().unwrap_or(0);
                if selected < app.offices.len() {
                    let office_name = app.offices[selected].name.clone();
                    app.rename = Some((RenameTarget::Office(selected), office_name));
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
                if app.settings_selected > 0 { app.settings_selected -= 1; }
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
            KeyCode::Char('q') => return true,  // Quit in Jump Mode
            KeyCode::Esc => {
                app.jump_mode = JumpMode::None;
            }
            // Arrow keys or vim keys to switch focus
            KeyCode::Up | KeyCode::Char('w') => {
                app.focus = Focus::Topbar;
                app.jump_mode = JumpMode::None;
            }
            KeyCode::Left | KeyCode::Char('a') => {
                app.focus = Focus::Sidebar;
                app.jump_mode = JumpMode::None;
            }
            // Letter keys for jumping
            KeyCode::Char(c) if c.is_ascii_lowercase() => {
                app.handle_jump_selection(c);
            }
            _ => {}
        }
        return false;
    }

    // Rename mode intercepts all keys
    if app.rename.is_some() {
        match key.code {
            KeyCode::Enter     => app.commit_rename(),
            KeyCode::Esc       => app.cancel_rename(),
            KeyCode::Backspace => { if let Some((_, buf)) = &mut app.rename { buf.pop(); } }
            KeyCode::Char(c) if !ctrl => { if let Some((_, buf)) = &mut app.rename { buf.push(c); } }
            _ => {}
        }
        return false;
    }

    // Ctrl+Z suspend
    if ctrl && key.code == KeyCode::Char('z') && app.focus != Focus::Content {
        #[cfg(unix)] {
            use std::io::Write;
            disable_raw_mode().ok();
            execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).ok();
            io::stdout().flush().ok();
            
            // Send SIGTSTP to self
            unsafe { libc::kill(libc::getpid(), libc::SIGTSTP); }
            
            // After resume (fg) - reinitialize everything
            enable_raw_mode().ok();
            execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture).ok();
            io::stdout().flush().ok();
        }
        return false;
    }

    if key.code == KeyCode::Esc {
        match app.focus {
            Focus::Topbar  => { app.focus = Focus::Sidebar; }
            Focus::Sidebar => {}
            Focus::Content => {
                // Enter Jump Mode - can use arrows OR letters
                app.jump_mode = JumpMode::Active;
            }
        }
        return false;
    }

    // Alt+1-9: switch tab
    if alt {
        if let KeyCode::Char(c) = key.code {
            if let Some(n) = c.to_digit(10) {
                let idx = (n as usize).saturating_sub(1);
                let i = app.selected();
                if idx < app.offices[app.current_office].desks[i].tabs.len() {
                    app.offices[app.current_office].desks[i].active_tab = idx;
                    app.spawn_active_pty();
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
                app.office_selector.list_state.select(Some(app.current_office));
            }
            KeyCode::Char('n') => app.new_desk(),
            KeyCode::Char('x') => app.close_desk(),
            KeyCode::Char('r') => { let i = app.selected(); app.begin_rename_desk(i); }
            KeyCode::Char('s') => { app.show_settings = true; }
            KeyCode::Up        => app.nav(-1),
            KeyCode::Down      => app.nav(1),
            KeyCode::Enter     => { app.focus = Focus::Content; app.spawn_active_pty(); }
            _ => {}
        },
        Focus::Topbar => match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Left  => {
                let i = app.selected();
                let at = app.offices[app.current_office].desks[i].active_tab;
                if at > 0 { app.offices[app.current_office].desks[i].active_tab = at - 1; app.spawn_active_pty(); }
            }
            KeyCode::Right => {
                let i = app.selected();
                let len = app.offices[app.current_office].desks[i].tabs.len();
                let at = app.offices[app.current_office].desks[i].active_tab;
                if at + 1 < len { app.offices[app.current_office].desks[i].active_tab = at + 1; app.spawn_active_pty(); }
            }
            KeyCode::Char('n') => { app.cur_desk_mut().new_tab(); app.spawn_active_pty(); app.dirty = true; }
            KeyCode::Char('x') => { app.cur_desk_mut().close_tab(); app.dirty = true; }
            KeyCode::Char('r') => app.begin_rename_tab(),
            KeyCode::Enter     => { app.focus = Focus::Content; app.spawn_active_pty(); }
            _ => {}
        },
        Focus::Content => {
            let shift = key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT);
            // Shift+PageUp/Down = scrollback
            if shift && key.code == KeyCode::PageUp   { app.pty_scroll(5);  return false; }
            if shift && key.code == KeyCode::PageDown  { app.pty_scroll(-5); return false; }
            let bytes: Vec<u8> = match key.code {
                KeyCode::Enter     => b"\r".to_vec(),
                KeyCode::Backspace => vec![0x7f],
                KeyCode::Tab       => b"\t".to_vec(),
                KeyCode::Up        => b"\x1b[A".to_vec(),
                KeyCode::Down      => b"\x1b[B".to_vec(),
                KeyCode::Right     => b"\x1b[C".to_vec(),
                KeyCode::Left      => b"\x1b[D".to_vec(),
                KeyCode::Char(c)   => {
                    if ctrl { vec![(c as u8).wrapping_sub(b'a').wrapping_add(1)] }
                    else { c.to_string().into_bytes() }
                }
                _ => vec![],
            };
            if !bytes.is_empty() { app.pty_write(&bytes); }
        }
    }
    false
}
