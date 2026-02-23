use crate::client::app::{App, Focus, RenameTarget};
use crate::terminal_provider::CursorShape;
use crate::theme::ThemeColors;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

fn border_style(t: &ThemeColors, active: bool) -> Style {
    if t.follow_terminal {
        if active {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }
    } else if active {
        Style::default().fg(t.accent())
    } else {
        Style::default().fg(t.border())
    }
}
fn title_style(t: &ThemeColors, active: bool) -> Style {
    if t.follow_terminal {
        if active {
            Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().add_modifier(Modifier::DIM)
        }
    } else if active {
        Style::default().fg(t.accent()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(t.fg_dim())
    }
}

fn border_type(t: &ThemeColors, active: bool) -> BorderType {
    if t.follow_terminal && active {
        BorderType::Thick
    } else {
        BorderType::Plain
    }
}

fn display_width(text: &str) -> u16 {
    UnicodeWidthStr::width(text) as u16
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let t = app.theme.clone();
    f.render_widget(
        Block::default().style(Style::default().bg(t.bg())),
        f.area(),
    );

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(0)])
        .split(root[0]);

    let main_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(cols[1]);

    let tr = main_rows[1].height.saturating_sub(2);
    let tc = main_rows[1].width.saturating_sub(2);
    // Update dimensions for spawn (no resize triggered here)
    app.term_rows = tr;
    app.term_cols = tc;

    app.sidebar_area = cols[0];
    app.topbar_area = main_rows[0];
    app.content_area = main_rows[1];

    draw_sidebar(f, app, cols[0], &t);
    draw_topbar(f, app, main_rows[0], &t);
    draw_terminal(f, app, main_rows[1], &t);
    draw_statusbar(f, app, root[1], &t);

    // Keep rename popup on top of all overlays so it is always visible.
    if app.rename.is_some() {
        draw_rename_popup(f, app, &t);
    }
}

fn draw_statusbar(f: &mut Frame, app: &App, area: Rect, t: &ThemeColors) {
    let keys: &[(&str, &str)] = if app.rename.is_some() {
        &[("Enter", "Confirm"), ("Esc", "Cancel")]
    } else {
        &[
            ("Ctrl+Q", "Quit"),
            ("PgUp/Dn", "Scroll"),
            ("Alt+N", "Tab"),
        ]
    };
    let mut spans: Vec<Span> = vec![Span::raw(" ")];
    for (key, desc) in keys {
        let key_style = if t.follow_terminal {
            Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default()
                .fg(t.bg())
                .bg(t.accent())
                .add_modifier(Modifier::BOLD)
        };
        spans.push(Span::styled(format!(" {key} "), key_style));
        if !desc.is_empty() {
            let desc_style = if t.follow_terminal {
                Style::default().add_modifier(Modifier::DIM)
            } else {
                Style::default().fg(t.fg_dim())
            };
            spans.push(Span::styled(format!(" {desc}  "), desc_style));
        } else {
            spans.push(Span::raw("  "));
        }
    }
    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(t.surface())),
        area,
    );

    // Update available notice on the right
    if let Some(ref ver) = app.update_available {
        let notice = format!(" ↑ Update available: {} — mato.sh ", ver);
        let w = notice.len() as u16;
        if w < area.width {
            f.render_widget(
                Paragraph::new(Span::styled(
                    notice,
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Rect {
                    x: area.x + area.width - w,
                    y: area.y,
                    width: w,
                    height: 1,
                },
            );
        }
    }
}

fn draw_sidebar(f: &mut Frame, app: &mut App, area: Rect, t: &ThemeColors) {
    let active = app.focus == Focus::Sidebar;
    let can_close = app.desks.len() > 1;

    let selected_desk_idx = app.selected();
    let items: Vec<ListItem> = app
        .desks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let sel = app.list_state.selected() == Some(i);

            // Active desk should never show spinner in sidebar.
            let has_spinner = if i == selected_desk_idx {
                false
            } else {
                task.tabs
                    .iter()
                    .any(|tab| app.active_tabs.contains(&tab.id))
            };

            let name = if has_spinner {
                format!("{} {}", task.name, app.get_spinner())
            } else {
                task.name.clone()
            };
            let item_style = if t.follow_terminal {
                if sel {
                    Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default().add_modifier(Modifier::DIM)
                }
            } else {
                Style::default().fg(if sel { t.fg() } else { t.fg_dim() })
            };
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        if sel { " ▶ " } else { "   " },
                        Style::default().fg(t.accent()),
                    ),
                    Span::styled(name, item_style),
                ]),
                Line::from(""),
            ])
            .style(Style::default().bg(if sel { t.sel_bg() } else { t.surface() }))
        })
        .collect();

    app.sidebar_list_area = area;

    // [+ New] button in bottom border
    let plus_label = " + New ";
    let plus_style = if t.follow_terminal {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(t.accent2())
    };
    // Position: centered in bottom border
    let plus_w = display_width(plus_label);
    let plus_x = area.x + (area.width.saturating_sub(plus_w)) / 2;
    app.new_desk_area = Rect {
        x: plus_x,
        y: area.y + area.height.saturating_sub(2),
        width: plus_w,
        height: 2, // cover border row + row above for easier touch
    };

    f.render_stateful_widget(
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(border_type(t, active))
                .title(Span::styled(" Desks ", title_style(t, active)))
                .title_bottom(
                    Line::from(Span::styled(plus_label, plus_style)).centered(),
                )
                .border_style(border_style(t, active))
                .style(Style::default().bg(t.surface())),
        ),
        area,
        &mut app.list_state,
    );

    // Overlay [×] close buttons for each visible desk
    app.desk_close_areas.clear();
    let inner_x = area.x + 1;
    let inner_y = area.y + 1;
    let inner_w = area.width.saturating_sub(2);
    let inner_h = area.height.saturating_sub(2);
    let offset = app.list_state.offset();
    let close_style = if t.follow_terminal {
        Style::default().add_modifier(Modifier::DIM)
    } else {
        Style::default().fg(t.fg_dim())
    };

    // Each desk item is 2 rows tall (name + spacer)
    for i in 0..app.desks.len() {
        let vis_row = (i as i32 - offset as i32) * 2; // 2 rows per item
        if can_close && vis_row >= 0 && (vis_row as u16) < inner_h {
            let close_rect = Rect {
                x: inner_x + inner_w.saturating_sub(4),
                y: inner_y + vis_row as u16,
                width: 4,
                height: 2, // cover both rows of desk item for easier touch
            };
            app.desk_close_areas.push(close_rect);
            f.render_widget(
                Paragraph::new(Span::styled(" × ", close_style)),
                close_rect,
            );
        } else {
            app.desk_close_areas.push(Rect::default());
        }
    }
}

fn draw_topbar(f: &mut Frame, app: &mut App, area: Rect, t: &ThemeColors) {
    let active = app.focus == Focus::Topbar;
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(border_type(t, active))
            .border_style(border_style(t, active))
            .style(Style::default().bg(t.surface())),
        area,
    );

    let task = &app.desks[app.selected()];
    let inner_w = area.width.saturating_sub(2);
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: inner_w,
        height: 1,
    };

    let at = task.active_tab;
    if at < app.tab_scroll {
        app.tab_scroll = at;
    }
    let plus = "  ＋  ";
    let plus_w = display_width(plus);
    let tab_widths: Vec<u16> = task
        .tabs
        .iter()
        .map(|tb| display_width(&format!("  {}  ", tb.name)) + 1)
        .collect();
    loop {
        let mut used = plus_w;
        let mut last_visible = app.tab_scroll;
        for (i, w) in tab_widths
            .iter()
            .enumerate()
            .take(task.tabs.len())
            .skip(app.tab_scroll)
        {
            if used + *w > inner_w {
                break;
            }
            used += *w;
            last_visible = i;
        }
        if at <= last_visible || app.tab_scroll >= at {
            break;
        }
        app.tab_scroll += 1;
    }

    app.tab_areas.clear();
    app.tab_area_tab_indices.clear();
    app.tab_close_areas.clear();
    let mut x = inner.x;
    let mut spans: Vec<Span> = vec![];
    let can_close_tab = task.tabs.len() > 1;

    if app.tab_scroll > 0 {
        spans.push(Span::styled("‹ ", Style::default().fg(t.accent2())));
        x += 2;
    }

    let close_str = " × ";
    let close_w = display_width(close_str);
    let close_style = if t.follow_terminal {
        Style::default().add_modifier(Modifier::DIM)
    } else {
        Style::default().fg(t.fg_dim())
    };

    let mut available = inner_w.saturating_sub(plus_w + if app.tab_scroll > 0 { 2 } else { 0 });
    for i in app.tab_scroll..task.tabs.len() {
        let tab = &task.tabs[i];
        let is_current_tab = i == task.active_tab;
        let is_active_tab = app.active_tabs.contains(&tab.id);

        // Only show spinner if: active AND not current tab
        let show_spinner = is_active_tab && !is_current_tab;

        let label = if show_spinner {
            format!("  {} {}  ", tab.name, app.get_spinner())
        } else {
            format!("  {}  ", tab.name)
        };
        let w = display_width(&label);
        let extra = if can_close_tab { close_w + 1 } else { 0 };
        if w + extra + 1 > available {
            break;
        }
        let style = if is_current_tab {
            if t.follow_terminal {
                Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default()
                    .fg(t.bg())
                    .bg(t.accent())
                    .add_modifier(Modifier::BOLD)
            }
        } else if t.follow_terminal {
            if is_active_tab {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            }
        } else {
            Style::default().fg(t.fg_dim()).bg(t.surface())
        };
        spans.push(Span::styled(label, style));
        app.tab_areas.push(Rect {
            x,
            y: area.y,
            width: w,
            height: area.height,
        });
        app.tab_area_tab_indices.push(i);
        if can_close_tab {
            spans.push(Span::styled(close_str, close_style));
            app.tab_close_areas.push(Rect {
                x: x + w,
                y: area.y,
                width: close_w,
                height: area.height,
            });
            x += w + close_w + 1;
            available = available.saturating_sub(w + close_w + 1);
        } else {
            app.tab_close_areas.push(Rect::default());
            x += w + 1;
            available = available.saturating_sub(w + 1);
        }
        spans.push(Span::raw(" "));
    }

    let last_rendered = app.tab_scroll + app.tab_areas.len();
    if last_rendered < task.tabs.len() {
        let arrow = " ›";
        spans.push(Span::styled(arrow, Style::default().fg(t.accent2())));
        x += display_width(arrow);
    }

    let plus_style = if t.follow_terminal {
        Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        Style::default().fg(t.accent2())
    };
    spans.push(Span::styled(plus, plus_style));
    app.new_tab_area = Rect {
        x,
        y: area.y,
        width: plus_w,
        height: area.height,
    };

    let daemon_status = if app.daemon_connected {
        " ✓ "
    } else if app.spinner_frame.is_multiple_of(2) {
        " · "
    } else {
        " • "
    };
    let status_w = daemon_status.len() as u16;
    let status_x = inner.x + inner_w - status_w;
    if status_x > x + plus_w {
        let daemon_style = if t.follow_terminal {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.accent2())
        };
        f.render_widget(
            Paragraph::new(Span::styled(daemon_status, daemon_style)),
            Rect {
                x: status_x,
                y: inner.y,
                width: status_w,
                height: 1,
            },
        );
    }

    f.render_widget(Paragraph::new(Line::from(spans)), inner);
}

fn draw_terminal(f: &mut Frame, app: &mut App, area: Rect, t: &ThemeColors) {
    let active = app.focus == Focus::Content;
    let task = &app.desks[app.selected()];
    let tab = task.active_tab_ref();

    let term_bg = t.bg();
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(border_type(t, active))
            .title(Span::styled(
                format!(
                    " {} ",
                    match app.terminal_titles.get(&tab.id) {
                        Some(term_title) if !term_title.is_empty() =>
                            format!("{} : {}", tab.name, term_title),
                        _ => tab.name.clone(),
                    }
                ),
                title_style(t, active),
            ))
            .border_style(border_style(t, active))
            .style(Style::default().bg(term_bg)),
        area,
    );
    let (ix, iy, iw, ih) = (
        area.x + 1,
        area.y + 1,
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    );
    let screen = tab.provider.get_screen(ih, iw);
    let screen_rows = (screen.lines.len() as u16).min(ih);
    // Top-aligned viewport to avoid startup offset when
    // PTY rows temporarily lag behind UI rows.
    let row_base: u16 = 0;

    if screen.bell {
        app.pending_bell = true;
    }

    let buf = f.buffer_mut();
    let bg_style = Style::default().bg(term_bg);

    for row_idx in 0..ih {
        let src_row = if row_idx < row_base {
            None
        } else {
            Some((row_idx - row_base) as usize)
        };
        let by = iy + row_idx;
        if let Some(line) = src_row.and_then(|r| screen.lines.get(r)) {
            let mut bx = ix;
            let bx_end = ix + iw;
            for cell in &line.cells {
                if bx >= bx_end {
                    break;
                }
                if cell.display_width == 0 {
                    continue;
                }
                if let Some(buf_cell) = buf.cell_mut((bx, by)) {
                    // Build style with bitwise modifier accumulation
                    let mut style = Style::default();
                    if let Some(fg) = cell.fg {
                        style = style.fg(fg);
                    }
                    if let Some(bg) = cell.bg {
                        style = style.bg(bg);
                    }
                    let mut mods = Modifier::empty();
                    if cell.bold { mods |= Modifier::BOLD; }
                    if cell.italic { mods |= Modifier::ITALIC; }
                    if cell.underline {
                        mods |= Modifier::UNDERLINED;
                        if let Some(uc) = cell.underline_color {
                            style = style.underline_color(uc);
                        }
                    }
                    if cell.dim { mods |= Modifier::DIM; }
                    if cell.reverse { mods |= Modifier::REVERSED; }
                    if cell.strikethrough { mods |= Modifier::CROSSED_OUT; }
                    if cell.hidden { mods |= Modifier::HIDDEN; }
                    if !mods.is_empty() {
                        style = style.add_modifier(mods);
                    }
                    buf_cell.set_style(style);
                    if cell.ch == '\0' {
                        buf_cell.set_char(' ');
                    } else if let Some(ref zw) = cell.zerowidth {
                        let mut sym = cell.ch.to_string();
                        for &c in zw {
                            sym.push(c);
                        }
                        buf_cell.set_symbol(&sym);
                    } else {
                        buf_cell.set_char(cell.ch);
                    }
                    // Wide chars: reset following continuation cells
                    if cell.display_width > 1 {
                        for dx in 1..cell.display_width as u16 {
                            let cx = bx + dx;
                            if cx < bx_end {
                                if let Some(next_cell) = buf.cell_mut((cx, by)) {
                                    next_cell.reset();
                                }
                            }
                        }
                    }
                }
                bx += cell.display_width as u16;
            }
            // Pad remaining columns with terminal background
            while bx < bx_end {
                if let Some(buf_cell) = buf.cell_mut((bx, by)) {
                    buf_cell.set_char(' ');
                    buf_cell.set_style(bg_style);
                }
                bx += 1;
            }
        } else {
            // Empty row — fill with background
            for col in 0..iw {
                if let Some(buf_cell) = buf.cell_mut((ix + col, by)) {
                    buf_cell.set_char(' ');
                    buf_cell.set_style(bg_style);
                }
            }
        }
    }

    let (cr, cc) = screen.cursor;
    // Hardware cursor is always hidden (terminal.hide_cursor at startup).
    // We use a software cursor overlay rendered in the buffer instead.
    // For Hidden cursor shape (e.g. Claude Code), skip the overlay entirely —
    // the inner TUI app renders its own visual cursor via INVERSE text.
    if ih > 0 && iw > 0 && screen.cursor_shape != CursorShape::Hidden {
        let cursor_row = cr.min(screen_rows.saturating_sub(1));
        let cursor_col = cc.min(iw.saturating_sub(1));
        let visual_cc = screen
            .lines
            .get(cursor_row as usize)
            .map(|line| {
                line.cells
                    .iter()
                    .take(cursor_col as usize)
                    .map(|cell| usize::from(cell.display_width))
                    .sum::<usize>() as u16
            })
            .unwrap_or(cursor_col);
        let cursor_x = ix + visual_cc.min(iw.saturating_sub(1));
        let cursor_y = iy + row_base + cursor_row;

        // Software cursor overlay: render a visible caret in the buffer.
        let line = screen.lines.get(cursor_row as usize);
        let mut glyph = " ".to_string();
        let mut caret_style = Style::default()
            .bg(term_bg)
            .add_modifier(Modifier::REVERSED);
        if let Some(line) = line {
            let mut idx = cc as usize;
            if idx >= line.cells.len() && !line.cells.is_empty() {
                idx = line.cells.len() - 1;
            }
            let mut cell = line.cells.get(idx);
            if matches!(cell, Some(c) if c.ch == '\0') && idx > 0 {
                cell = line.cells.get(idx - 1);
            }
            if let Some(cell) = cell {
                if cell.ch != '\0' {
                    glyph = cell.ch.to_string();
                }
                if let Some(fg) = cell.fg {
                    caret_style = caret_style.fg(fg);
                }
                if let Some(bg) = cell.bg {
                    caret_style = caret_style.bg(bg);
                }
                if cell.bold {
                    caret_style = caret_style.add_modifier(Modifier::BOLD);
                }
                if cell.italic {
                    caret_style = caret_style.add_modifier(Modifier::ITALIC);
                }
                if cell.underline {
                    caret_style = caret_style.add_modifier(Modifier::UNDERLINED);
                }
                caret_style = caret_style.add_modifier(Modifier::REVERSED);
            }
        }
        f.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(glyph, caret_style)])),
            Rect {
                x: cursor_x,
                y: cursor_y,
                width: 1,
                height: 1,
            },
        );
    }
}

fn draw_rename_popup(f: &mut Frame, app: &App, t: &ThemeColors) {
    let Some((target, buf)) = &app.rename else {
        return;
    };
    let label = match target {
        RenameTarget::Desk(_) => " Rename Desk ",
        RenameTarget::Tab(_, _) => " Rename Tab ",
    };
    let area = f.area();
    let w = 40u16.min(area.width);
    let popup = Rect {
        x: (area.width.saturating_sub(w)) / 2,
        y: area.height / 2 - 2,
        width: w,
        height: 3,
    };
    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(buf.clone(), Style::default().fg(t.fg())),
            Span::styled("█", Style::default().fg(t.accent())),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    label,
                    Style::default().fg(t.accent()).add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(t.accent()))
                .style(Style::default().bg(t.surface())),
        ),
        popup,
    );
}
