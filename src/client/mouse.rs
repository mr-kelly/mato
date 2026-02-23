use crossterm::event::{MouseButton, MouseEventKind};
use ratatui::layout::Rect;

use super::app::{App, Focus};

pub fn handle_mouse(app: &mut App, me: crossterm::event::MouseEvent) {
    let (col, row) = (me.column, me.row);
    fn in_rect(col: u16, row: u16, r: Rect) -> bool {
        col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height
    }

    // Mouse passthrough to PTY when in content focus
    if matches!(app.focus, Focus::Content) && in_rect(col, row, app.content_area) {
        let tx = col.saturating_sub(app.content_area.x + 1) + 1;
        let ty = row.saturating_sub(app.content_area.y + 1) + 1;
        let mouse_mode = app.pty_mouse_mode_enabled();
        match me.kind {
            MouseEventKind::ScrollUp => {
                if mouse_mode {
                    app.pty_write(format!("\x1b[<64;{};{}M", tx, ty).as_bytes());
                }
                return;
            }
            MouseEventKind::ScrollDown => {
                if mouse_mode {
                    app.pty_write(format!("\x1b[<65;{};{}M", tx, ty).as_bytes());
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
                    let tab_indices = app.tab_area_tab_indices.clone();
                    for (i, ta) in tab_areas.iter().enumerate() {
                        if in_rect(col, row, *ta) {
                            let ti = app.selected();
                            // tab_area_tab_indices maps visual position → real tab index
                            let real_tab_idx = tab_indices.get(i).copied().unwrap_or(i);
                            let is_double = app
                                .last_click
                                .as_ref()
                                .map(|&(lc, lr, ref t)| {
                                    lc == col && lr == row && t.elapsed().as_millis() < 400
                                })
                                .unwrap_or(false);
                            if is_double
                                && app.offices[app.current_office].desks[ti].active_tab
                                    == real_tab_idx
                            {
                                app.begin_rename_tab();
                            } else {
                                app.offices[app.current_office].desks[ti].active_tab = real_tab_idx;
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
                let prev = app.tab_scroll;
                app.tab_scroll = app.tab_scroll.saturating_sub(1);
                if app.tab_scroll != prev {
                    app.dirty = true;
                }
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
                let prev = app.tab_scroll;
                if app.tab_scroll < max {
                    app.tab_scroll += 1;
                }
                if app.tab_scroll != prev {
                    app.dirty = true;
                }
            }
        }
        MouseEventKind::ScrollLeft => {
            if in_rect(col, row, app.topbar_area) {
                let prev = app.tab_scroll;
                app.tab_scroll = app.tab_scroll.saturating_sub(1);
                if app.tab_scroll != prev {
                    app.dirty = true;
                }
            }
        }
        MouseEventKind::ScrollRight => {
            if in_rect(col, row, app.topbar_area) {
                let max = app.offices[app.current_office].desks[app.selected()]
                    .tabs
                    .len()
                    .saturating_sub(1);
                let prev = app.tab_scroll;
                if app.tab_scroll < max {
                    app.tab_scroll += 1;
                }
                if app.tab_scroll != prev {
                    app.dirty = true;
                }
            }
        }
        _ => {}
    }
}
