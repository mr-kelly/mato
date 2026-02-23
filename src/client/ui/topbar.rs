use crate::client::app::{App, Focus, JumpMode};
use crate::theme::ThemeColors;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(super) fn draw_topbar(f: &mut Frame, app: &mut App, area: Rect, t: &ThemeColors) {
    let active = app.focus == Focus::Topbar;
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(super::border_type(t, active))
            .border_style(super::border_style(t, active))
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
    let plus_w = super::display_width(plus);
    let tab_widths: Vec<u16> = task
        .tabs
        .iter()
        .map(|tb| super::display_width(&format!("  {}  ", tb.name)) + 1)
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
        let w = super::display_width(&label);
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

pub(super) fn draw_statusbar(f: &mut Frame, app: &App, area: Rect, t: &ThemeColors) {
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
    } else if app.desk_delete_confirm.is_some() {
        &[("y/Enter", "Yes"), ("n/Esc", "No")]
    } else if app.rename.is_some() {
        &[("Enter", "Confirm"), ("Esc", "Cancel")]
    } else if let JumpMode::Active = app.jump_mode {
        // In Jump Mode, always show explicit focus targets as separate keys.
        match app.focus {
            Focus::Content => &[
                ("a-z/A-Z/0-9", "Jump"),
                ("c", "Copy Mode"),
                ("r", "Restart Terminal"),
                ("←", "Focus Sidebar"),
                ("↑", "Focus Tabbar"),
                ("q", "Quit"),
                ("Esc", "Cancel"),
            ],
            Focus::Topbar => &[
                ("a-z/A-Z/0-9", "Jump"),
                ("r", "Rename"),
                ("←", "Focus Sidebar"),
                ("↓", "Focus Content"),
                ("q", "Quit"),
                ("Esc", "Cancel"),
            ],
            Focus::Sidebar => &[
                ("a-z/A-Z/0-9", "Jump"),
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
