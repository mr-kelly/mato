use std::io;
use crossterm::{
    event::{KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crate::client::app::{App, EscMode, Focus};

pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    if key.code == KeyCode::Esc && key.kind == KeyEventKind::Repeat { return false; }
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

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

    // Alt+1-9: Quick tab switching
    if alt && matches!(key.code, KeyCode::Char('1'..='9')) {
        if let KeyCode::Char(c) = key.code {
            let idx = (c as u8 - b'1') as usize;
            let task = app.cur_task_mut();
            if idx < task.tabs.len() {
                task.active_tab = idx;
                app.tab_scroll = idx.saturating_sub(5); // Center the view
            }
        }
        return false;
    }

    // Ctrl+Z suspend
    if ctrl && key.code == KeyCode::Char('z') && app.focus != Focus::Content {
        #[cfg(unix)] {
            disable_raw_mode().ok();
            execute!(io::stdout(), LeaveAlternateScreen).ok();
            unsafe { libc::kill(libc::getpid(), libc::SIGTSTP); }
            enable_raw_mode().ok();
            execute!(io::stdout(), EnterAlternateScreen).ok();
        }
        return false;
    }

    if key.code == KeyCode::Esc {
        match app.focus {
            Focus::Topbar  => { app.focus = Focus::Sidebar; }
            Focus::Sidebar => {}
            Focus::Content => {
                if app.esc_mode != EscMode::Pending { app.esc_mode = EscMode::Pending; }
            }
        }
        return false;
    }

    if app.esc_mode == EscMode::Pending {
        app.esc_mode = EscMode::None;
        match key.code {
            KeyCode::Up   | KeyCode::Char('w') => { app.focus = Focus::Topbar;  return false; }
            KeyCode::Left | KeyCode::Char('a') => { app.focus = Focus::Sidebar; return false; }
            _ => {}
        }
    }

    // Ctrl+PageUp/PageDown: Switch tabs (works in any focus)
    if ctrl {
        match key.code {
            KeyCode::PageUp => {
                let i = app.selected();
                let at = app.tasks[i].active_tab;
                if at > 0 {
                    app.tasks[i].active_tab = at - 1;
                    app.spawn_active_pty();
                }
                return false;
            }
            KeyCode::PageDown => {
                let i = app.selected();
                let len = app.tasks[i].tabs.len();
                let at = app.tasks[i].active_tab;
                if at + 1 < len {
                    app.tasks[i].active_tab = at + 1;
                    app.spawn_active_pty();
                }
                return false;
            }
            _ => {}
        }
    }

    match app.focus {
        Focus::Sidebar => match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('n') => app.new_task(),
            KeyCode::Char('x') => app.close_task(),
            KeyCode::Char('r') => { let i = app.selected(); app.begin_rename_task(i); }
            KeyCode::Up        => app.nav(-1),
            KeyCode::Down      => app.nav(1),
            KeyCode::Enter     => { app.focus = Focus::Content; app.spawn_active_pty(); }
            _ => {}
        },
        Focus::Topbar => match key.code {
            KeyCode::Left  => {
                let i = app.selected();
                let at = app.tasks[i].active_tab;
                if at > 0 { app.tasks[i].active_tab = at - 1; app.spawn_active_pty(); }
            }
            KeyCode::Right => {
                let i = app.selected();
                let len = app.tasks[i].tabs.len();
                let at = app.tasks[i].active_tab;
                if at + 1 < len { app.tasks[i].active_tab = at + 1; app.spawn_active_pty(); }
            }
            KeyCode::Char('t') => { app.cur_task_mut().new_tab(); app.spawn_active_pty(); app.dirty = true; }
            KeyCode::Char('w') => { app.cur_task_mut().close_tab(); app.dirty = true; }
            KeyCode::Char('r') => app.begin_rename_tab(),
            KeyCode::Enter     => { app.focus = Focus::Content; app.spawn_active_pty(); }
            _ => {}
        },
        Focus::Content => {
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
