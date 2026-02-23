use crate::client::app::App;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};

pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    if key.code == KeyCode::Esc && key.kind == KeyEventKind::Repeat {
        return false;
    }
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

    // Emergency exit when daemon connection is unhealthy.
    if !app.daemon_connected
        && (matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q'))
            || (ctrl && key.code == KeyCode::Char('c')))
    {
        return true;
    }

    // Rename mode intercepts all keys
    if app.rename.is_some() {
        match key.code {
            KeyCode::Enter => app.commit_rename(),
            KeyCode::Esc => app.cancel_rename(),
            KeyCode::Backspace => {
                if let Some((_, buf)) = &mut app.rename {
                    buf.pop();
                }
            }
            KeyCode::Char(c) if !ctrl => {
                if let Some((_, buf)) = &mut app.rename {
                    buf.push(c);
                }
            }
            _ => {}
        }
        return false;
    }

    // Ctrl+Q: quit
    if ctrl && key.code == KeyCode::Char('q') {
        return true;
    }

    // PageUp/PageDown: scroll terminal scrollback
    if key.code == KeyCode::PageUp {
        app.pty_scroll(5);
        return false;
    }
    if key.code == KeyCode::PageDown {
        app.pty_scroll(-5);
        return false;
    }

    // Alt+1-9: switch tab
    if alt {
        if let KeyCode::Char(c) = key.code {
            if let Some(n) = c.to_digit(10) {
                let idx = (n as usize).saturating_sub(1);
                let i = app.selected();
                if idx < app.desks[i].tabs.len() {
                    app.desks[i].active_tab = idx;
                    app.mark_tab_switch();
                    app.spawn_active_pty();
                }
                return false;
            }
        }
    }

    // All other keys go to the terminal
    let bytes = encode_content_key(&key);
    if !bytes.is_empty() {
        app.pty_write(&bytes);
    }
    false
}

fn encode_content_key(key: &crossterm::event::KeyEvent) -> Vec<u8> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let mut bytes: Vec<u8> = match key.code {
        KeyCode::Esc => vec![0x1b],
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
