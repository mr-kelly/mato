use crate::client::app::{App, Focus, JumpMode, RenameTarget, JUMP_LABELS};
use crate::terminal_provider::CursorShape;
use crate::theme::{ThemeColors, BUILTIN_THEMES};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
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

    if app.copy_mode {
        app.sidebar_area = Rect::default();
        app.topbar_area = Rect::default();
        app.sidebar_list_area = Rect::default();
        app.tab_areas.clear();
        app.tab_area_tab_indices.clear();
        app.new_desk_area = Rect::default();
        app.new_tab_area = Rect::default();
        app.content_area = root[0];

        let tr = root[0].height.saturating_sub(2);
        let tc = root[0].width.saturating_sub(2);
        app.term_rows = tr;
        app.term_cols = tc;
        draw_terminal(f, app, root[0], &t);
    } else {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(24), Constraint::Min(0)])
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
    }
    draw_statusbar(f, app, root[1], &t);

    if let JumpMode::Active = app.jump_mode {
        draw_jump_mode(f, app, &t);
    }
    if app.show_settings {
        draw_settings(f, app, &t);
    }
    if app.office_selector.active {
        draw_office_selector(f, app, &t);
    }
    if app.office_delete_confirm.is_some() {
        draw_office_delete_confirm(f, app, &t);
    }
    // Keep rename popup on top of all overlays so it is always visible.
    if app.rename.is_some() {
        draw_rename_popup(f, app, &t);
    }
}

fn draw_statusbar(f: &mut Frame, app: &App, area: Rect, t: &ThemeColors) {
    let focus_name = match app.focus {
        Focus::Sidebar => "Sidebar",
        Focus::Topbar => "Topbar",
        Focus::Content => "Content",
    };
    let keys: &[(&str, &str)] = if app.copy_mode {
        &[
            ("↑/k", "Scroll Up"),
            ("↓/j", "Scroll Down"),
            ("PgUp/PgDn", "Fast Scroll"),
            ("g/G", "Bottom/Top"),
            ("Esc/q", "Exit Copy"),
        ]
    } else if app.rename.is_some() {
        &[("Enter", "Confirm"), ("Esc", "Cancel")]
    } else if let JumpMode::Active = app.jump_mode {
        // In Jump Mode, always show explicit focus targets as separate keys.
        match app.focus {
            Focus::Content => &[
                ("a-z/A-Z", "Jump"),
                ("c", "Copy Mode"),
                ("r", "Restart Terminal"),
                ("←", "Focus Sidebar"),
                ("↑", "Focus Tabbar"),
                ("q", "Quit"),
                ("Esc", "Cancel"),
            ],
            Focus::Topbar => &[
                ("a-z/A-Z", "Jump"),
                ("r", "Rename"),
                ("←", "Focus Sidebar"),
                ("↓", "Focus Content"),
                ("q", "Quit"),
                ("Esc", "Cancel"),
            ],
            Focus::Sidebar => &[
                ("a-z/A-Z", "Jump"),
                ("r", "Rename"),
                ("→", "Focus Content"),
                ("↑", "Focus Tabbar"),
                ("Esc", "Cancel"),
            ],
        }
    } else {
        match app.focus {
            Focus::Sidebar => &[
                ("↑↓", "Navigate"),
                ("o", "Office"),
                ("n", "New Desk"),
                ("x", "Close"),
                ("r", "Rename"),
                ("s", "Settings"),
                ("Esc", "Jump"),
                ("q", "Quit"),
            ],
            Focus::Topbar => &[
                ("←→", "Switch Tab"),
                ("n", "New Tab"),
                ("x", "Close Tab"),
                ("r", "Rename"),
                ("Enter", "Focus"),
                ("Esc", "Jump"),
                ("q", "Quit"),
            ],
            Focus::Content => &[("Esc·Esc", "Jump"), ("keys→shell", "")],
        }
    };
    let focus_badge_style = if t.follow_terminal {
        Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        Style::default()
            .fg(t.bg())
            .bg(t.accent())
            .add_modifier(Modifier::BOLD)
    };
    let mut spans: Vec<Span> = vec![
        Span::raw(" "),
        Span::styled(format!(" Focus:{focus_name} "), focus_badge_style),
        Span::raw("  "),
    ];
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
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Office selector (top area)
    let office_name = &app.offices[app.current_office].name;
    let office_text = format!(" 🏢 {} ", office_name);
    let office_style = if active {
        if t.follow_terminal {
            Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default()
                .fg(t.fg())
                .bg(t.surface())
                .add_modifier(Modifier::BOLD)
        }
    } else if t.follow_terminal {
        Style::default().add_modifier(Modifier::DIM)
    } else {
        Style::default().fg(t.fg_dim()).bg(t.surface())
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(office_text, office_style)]))
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(border_type(t, active))
                    .border_style(border_style(t, active))
                    .style(Style::default().bg(t.surface())),
            ),
        rows[0],
    );
    app.new_desk_area = rows[0]; // Reuse this for office selector click area

    let selected_desk_idx = app.selected();
    let items: Vec<ListItem> = app.offices[app.current_office]
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
            ListItem::new(Line::from(vec![
                Span::styled(
                    if sel { " ▶ " } else { "   " },
                    Style::default().fg(t.accent()),
                ),
                Span::styled(name, item_style),
            ]))
            .style(Style::default().bg(if sel { t.sel_bg() } else { t.surface() }))
        })
        .collect();

    app.sidebar_list_area = rows[1];
    f.render_stateful_widget(
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(border_type(t, active))
                .title(Span::styled(" Desks ", title_style(t, active)))
                .border_style(border_style(t, active))
                .style(Style::default().bg(t.surface())),
        ),
        rows[1],
        &mut app.list_state,
    );
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

    let task = &app.offices[app.current_office].desks[app.selected()];
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
    let mut x = inner.x;
    let mut spans: Vec<Span> = vec![];

    if app.tab_scroll > 0 {
        spans.push(Span::styled("‹ ", Style::default().fg(t.accent2())));
        x += 2;
    }

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
        if w + 1 > available {
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
        spans.push(Span::raw(" "));
        app.tab_areas.push(Rect {
            x,
            y: inner.y,
            width: w,
            height: 1,
        });
        app.tab_area_tab_indices.push(i);
        x += w + 1;
        available = available.saturating_sub(w + 1);
    }

    let last_rendered = app.tab_scroll + app.tab_areas.len();
    if last_rendered < task.tabs.len() {
        spans.push(Span::styled(" ›", Style::default().fg(t.accent2())));
    }

    let plus_style = if t.follow_terminal {
        Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        Style::default().fg(t.accent2())
    };
    spans.push(Span::styled(plus, plus_style));
    app.new_tab_area = Rect {
        x,
        y: inner.y,
        width: plus_w,
        height: 1,
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
    let task = &app.offices[app.current_office].desks[app.selected()];
    let tab = task.active_tab_ref();

    let term_bg = t.bg();
    let (ix, iy, iw, ih) = if app.copy_mode {
        f.render_widget(
            Block::default().style(Style::default().bg(term_bg)),
            area,
        );
        (area.x, area.y, area.width, area.height)
    } else {
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
        (
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        )
    };
    let screen = tab.provider.get_screen(ih, iw);
    let screen_rows = (screen.lines.len() as u16).min(ih);
    let row_base = ih.saturating_sub(screen_rows);

    if screen.bell {
        app.pending_bell = true;
    }

    for row_idx in 0..ih {
        let src_row = if row_idx < row_base {
            None
        } else {
            Some((row_idx - row_base) as usize)
        };
        let spans: Vec<Span> = if let Some(line) = src_row.and_then(|r| screen.lines.get(r)) {
            let mut render_width = 0usize;
            let mut cells: Vec<Span> = line
                .cells
                .iter()
                .map(|cell| {
                    let mut style = Style::default();
                    if let Some(fg) = cell.fg {
                        style = style.fg(fg);
                    }
                    if let Some(bg) = cell.bg {
                        style = style.bg(bg);
                    }
                    if cell.bold {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if cell.italic {
                        style = style.add_modifier(Modifier::ITALIC);
                    }
                    if cell.underline {
                        style = style.add_modifier(Modifier::UNDERLINED);
                        if let Some(uc) = cell.underline_color {
                            style = style.underline_color(uc);
                        }
                    }
                    if cell.dim {
                        style = style.add_modifier(Modifier::DIM);
                    }
                    if cell.reverse {
                        style = style.add_modifier(Modifier::REVERSED);
                    }
                    if cell.strikethrough {
                        style = style.add_modifier(Modifier::CROSSED_OUT);
                    }
                    if cell.hidden {
                        style = style.add_modifier(Modifier::HIDDEN);
                    }
                    let glyph = if cell.ch == '\0' {
                        String::new()
                    } else {
                        let mut s = cell.ch.to_string();
                        if let Some(ref zw) = cell.zerowidth {
                            for &c in zw {
                                s.push(c);
                            }
                        }
                        s
                    };
                    render_width += usize::from(cell.display_width);
                    Span::styled(glyph, style)
                })
                .collect();
            // Pad to full visible width, accounting for zero-width spacer cells.
            if render_width < iw as usize {
                cells.push(Span::styled(
                    " ".repeat(iw as usize - render_width),
                    Style::default().bg(term_bg),
                ));
            }
            cells
        } else {
            vec![Span::styled(
                " ".repeat(iw as usize),
                Style::default().bg(term_bg),
            )]
        };
        f.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect {
                x: ix,
                y: iy + row_idx,
                width: iw,
                height: 1,
            },
        );
    }

    let (cr, cc) = screen.cursor;
    // Hardware cursor is always hidden (terminal.hide_cursor at startup).
    // We use a software cursor overlay rendered in the buffer instead.
    // For Hidden cursor shape (e.g. Claude Code), skip the overlay entirely —
    // the inner TUI app renders its own visual cursor via INVERSE text.
    if !app.copy_mode && ih > 0 && iw > 0 && screen.cursor_shape != CursorShape::Hidden {
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

fn draw_jump_mode(f: &mut Frame, app: &App, t: &ThemeColors) {
    let labels: Vec<char> = JUMP_LABELS.chars().collect();
    let jump_fg = if t.follow_terminal {
        Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        Style::default()
            .fg(t.bg())
            .bg(t.accent())
            .add_modifier(Modifier::BOLD)
    };
    let targets = app.jump_targets();
    for (idx, (kind, desk_idx, tab_idx)) in targets.iter().enumerate() {
        if idx >= labels.len() {
            break;
        }
        let label = labels[idx];
        match kind {
            't' => {
                let y = app.sidebar_list_area.y + 1 + *desk_idx as u16;
                let x = app.sidebar_list_area.x + 1;
                if y < app.sidebar_list_area.y + app.sidebar_list_area.height.saturating_sub(1) {
                    f.render_widget(
                        Paragraph::new(Span::styled(format!("[{}]", label), jump_fg)),
                        Rect {
                            x,
                            y,
                            width: 3,
                            height: 1,
                        },
                    );
                }
            }
            'b' => {
                if let Some(area_idx) = app.tab_area_tab_indices.iter().position(|i| *i == *tab_idx)
                {
                    if let Some(tab_area) = app.tab_areas.get(area_idx) {
                        let label_x = tab_area.x + tab_area.width.saturating_sub(3) / 2;
                        f.render_widget(
                            Paragraph::new(Span::styled(format!("[{}]", label), jump_fg)),
                            Rect {
                                x: label_x,
                                y: tab_area.y,
                                width: 3,
                                height: 1,
                            },
                        );
                    }
                }
            }
            _ => {}
        }
    }

    let help_area = Rect {
        x: app.content_area.x + 2,
        y: app.content_area.y + 2,
        width: 50,
        height: 4,
    };
    f.render_widget(Clear, help_area);

    // Help text varies by focus
    let help_line_3 = match app.focus {
        Focus::Content => " c CopyMode | r Restart | ← Sidebar | ↑ Tabbar | q quit | ESC cancel ",
        Focus::Topbar => " r Rename | ← Sidebar | ↓ Content | q quit | ESC cancel ",
        Focus::Sidebar => " r Rename | → Content | ↑ Tabbar | ESC cancel ",
    };

    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(" JUMP MODE ", jump_fg)),
            Line::from(Span::styled(
                " Press a-z or A-Z to jump ",
                Style::default().fg(t.fg()),
            )),
            Line::from(Span::styled(help_line_3, Style::default().fg(t.fg_dim()))),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(t.accent()))
                .style(Style::default().bg(if t.follow_terminal {
                    Color::Black
                } else {
                    t.surface()
                })),
        ),
        help_area,
    );
}

fn draw_rename_popup(f: &mut Frame, app: &App, t: &ThemeColors) {
    let Some((target, buf)) = &app.rename else {
        return;
    };
    let label = match target {
        RenameTarget::Desk(_) => " Rename Desk ",
        RenameTarget::Tab(_, _) => " Rename Tab ",
        RenameTarget::Office(_) => " Rename Office ",
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

pub fn draw_settings(f: &mut Frame, app: &mut App, t: &ThemeColors) {
    let area = f.area();
    let w = 50u16.min(area.width);
    let h = (BUILTIN_THEMES.len() as u16 + 6).min(area.height);
    let popup = Rect {
        x: (area.width.saturating_sub(w)) / 2,
        y: (area.height.saturating_sub(h)) / 2,
        width: w,
        height: h,
    };
    f.render_widget(Clear, popup);

    let items: Vec<ListItem> = BUILTIN_THEMES
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let sel = app.settings_selected == i;
            let label = if *name == "system" {
                "system (follow terminal)"
            } else {
                *name
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    if sel { " ▶ " } else { "   " },
                    Style::default().fg(t.accent()),
                ),
                Span::styled(
                    label,
                    Style::default().fg(if sel { t.fg() } else { t.fg_dim() }),
                ),
            ]))
            .style(Style::default().bg(if sel { t.sel_bg() } else { t.surface() }))
        })
        .collect();

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(app.settings_selected));

    f.render_stateful_widget(
        List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(
                        " Settings — Theme ",
                        Style::default().fg(t.accent()).add_modifier(Modifier::BOLD),
                    ))
                    .border_style(Style::default().fg(t.accent()))
                    .style(Style::default().bg(t.surface())),
            )
            .highlight_style(Style::default().bg(t.sel_bg())),
        popup,
        &mut list_state,
    );
}

fn draw_office_selector(f: &mut Frame, app: &mut App, t: &ThemeColors) {
    let area = f.area();
    let w = 50u16.min(area.width);
    let h = (app.offices.len() as u16 + 6).min(area.height);
    let popup = Rect {
        x: (area.width.saturating_sub(w)) / 2,
        y: (area.height.saturating_sub(h)) / 2,
        width: w,
        height: h,
    };
    f.render_widget(Clear, popup);

    let mut items: Vec<ListItem> = app
        .offices
        .iter()
        .enumerate()
        .map(|(i, office)| {
            let is_current = i == app.current_office;
            let prefix = if is_current { " ● " } else { "   " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(t.accent())),
                Span::styled(&office.name, Style::default().fg(t.fg())),
            ]))
        })
        .collect();

    items.push(ListItem::new(Line::from(vec![
        Span::styled("   ", Style::default()),
        Span::styled("＋ New Office", Style::default().fg(t.accent())),
    ])));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Switch Office ",
            Style::default().fg(t.accent()).add_modifier(Modifier::BOLD),
        ))
        .title_bottom(Line::from(vec![
            Span::styled(" Enter ", Style::default().fg(t.accent())),
            Span::styled("Select  ", Style::default().fg(t.fg_dim())),
            Span::styled("r ", Style::default().fg(t.accent())),
            Span::styled("Rename  ", Style::default().fg(t.fg_dim())),
            Span::styled("d ", Style::default().fg(t.accent())),
            Span::styled("Delete ", Style::default().fg(t.fg_dim())),
        ]))
        .border_style(Style::default().fg(t.accent()))
        .style(Style::default().bg(t.surface()));

    f.render_stateful_widget(
        List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(t.sel_bg()).add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ "),
        popup,
        &mut app.office_selector.list_state,
    );
}

fn draw_office_delete_confirm(f: &mut Frame, app: &App, t: &ThemeColors) {
    let Some(ref confirm) = app.office_delete_confirm else {
        return;
    };
    let office_name = &app.offices[confirm.office_idx].name;

    let area = f.area();
    let w = 60u16.min(area.width);
    let h = 9u16.min(area.height);
    let popup = Rect {
        x: (area.width.saturating_sub(w)) / 2,
        y: (area.height.saturating_sub(h)) / 2,
        width: w,
        height: h,
    };
    f.render_widget(Clear, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(popup);

    let warning = Paragraph::new(format!("⚠️  Delete office \"{}\"?", office_name))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );
    f.render_widget(warning, chunks[0]);

    let prompt = Paragraph::new(format!(
        "Type the office name to confirm:\n{}",
        confirm.input
    ))
    .alignment(Alignment::Center)
    .style(Style::default().fg(t.fg()));
    f.render_widget(prompt, chunks[1]);

    let help = Paragraph::new("Enter Confirm  │  Esc Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(t.fg_dim()));
    f.render_widget(help, chunks[2]);
}
