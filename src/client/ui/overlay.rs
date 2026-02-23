use crate::client::app::{App, Focus, RenameTarget};
use crate::theme::{ThemeColors, BUILTIN_THEMES};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use std::time::Duration;

pub(super) fn draw_jump_mode(f: &mut Frame, app: &App, t: &ThemeColors) {
    // 1. DIM THE BACKGROUND
    // Apply DIM modifier to every cell in the current buffer to suppress background noise
    let area = f.area();
    let buf = f.buffer_mut();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_style(cell.style().add_modifier(Modifier::DIM));
            }
        }
    }

    let labels = app.jump_labels();
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
                let x = app.sidebar_list_area.x + 1;
                let sidebar_offset = app.list_state.offset();
                if let Some(local_row) = desk_idx.checked_sub(sidebar_offset) {
                    let y = app.sidebar_list_area.y + 1 + local_row as u16;
                    if y < app.sidebar_list_area.y + app.sidebar_list_area.height.saturating_sub(1)
                    {
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
    let help_line_2 = match app.focus {
        Focus::Content => " Press letters or digits to jump (no c/r/q) ",
        Focus::Topbar | Focus::Sidebar => " Press letters or digits to jump (no r/q) ",
    };
    let help_line_3 = match app.focus {
        Focus::Content => " c CopyMode | r Restart | ← Sidebar | ↑ Tabbar | q quit | ESC cancel ",
        Focus::Topbar => " r Rename | ← Sidebar | ↓ Content | q quit | ESC cancel ",
        Focus::Sidebar => " r Rename | → Content | ↑ Tabbar | ESC cancel ",
    };

    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(" JUMP MODE ", jump_fg)),
            Line::from(Span::styled(help_line_2, Style::default().fg(t.fg()))),
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

pub(super) fn draw_rename_popup(f: &mut Frame, app: &App, t: &ThemeColors) {
    let Some(rename) = &app.rename else {
        return;
    };
    let label = match rename.target {
        RenameTarget::Desk(_) => " Rename Desk ",
        RenameTarget::Tab(_, _) => " Rename Tab ",
        RenameTarget::Office(_) => " Rename Office ",
    };
    let cursor_byte = rename.cursor_byte_index();
    let before = &rename.buffer[..cursor_byte];
    let after = &rename.buffer[cursor_byte..];
    let current_char_len = after.chars().next().map(|c| c.len_utf8()).unwrap_or(0);
    let current = &after[..current_char_len];
    let tail = &after[current_char_len..];
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
            Span::styled(before, Style::default().fg(t.fg())),
            Span::styled(
                if current.is_empty() { "█" } else { current },
                Style::default().fg(t.bg()).bg(t.accent()),
            ),
            Span::styled(tail, Style::default().fg(t.fg())),
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

pub(super) fn draw_settings(f: &mut Frame, app: &mut App, t: &ThemeColors) {
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

pub(super) fn draw_office_selector(f: &mut Frame, app: &mut App, t: &ThemeColors) {
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

pub(super) fn draw_office_delete_confirm(f: &mut Frame, app: &App, t: &ThemeColors) {
    let Some(ref confirm) = app.office_delete_confirm else {
        return;
    };
    if confirm.office_idx >= app.offices.len() {
        return;
    }
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

pub(super) fn draw_desk_delete_confirm(f: &mut Frame, app: &App, t: &ThemeColors) {
    let Some(ref confirm) = app.desk_delete_confirm else {
        return;
    };
    let office = &app.offices[app.current_office];
    if confirm.desk_idx >= office.desks.len() {
        return;
    }
    let desk_name = &office.desks[confirm.desk_idx].name;

    let area = f.area();
    let w = 58u16.min(area.width);
    let h = 7u16.min(area.height);
    let popup = Rect {
        x: (area.width.saturating_sub(w)) / 2,
        y: (area.height.saturating_sub(h)) / 2,
        width: w,
        height: h,
    };
    f.render_widget(Clear, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(2), Constraint::Length(2)])
        .split(popup);

    let warning = Paragraph::new(format!("⚠️  Delete desk \"{}\"?", desk_name))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );
    f.render_widget(warning, chunks[0]);

    let prompt = Paragraph::new("This will close all tabs and running PTYs in this desk.")
        .alignment(Alignment::Center)
        .style(Style::default().fg(t.fg()));
    f.render_widget(prompt, chunks[1]);

    let help = Paragraph::new("y / Enter = Yes   │   n / Esc = No")
        .alignment(Alignment::Center)
        .style(Style::default().fg(t.fg_dim()));
    f.render_widget(help, chunks[2]);
}

pub(super) fn draw_toast(f: &mut Frame, app: &App, t: &ThemeColors) {
    if let Some((msg, instant)) = &app.toast {
        let elapsed = instant.elapsed();
        if elapsed > Duration::from_secs(3) {
            return;
        }

        let area = f.area();
        let msg_len = (msg.len() + 4) as u16;
        let width = msg_len.min(area.width);
        let toast_area = Rect {
            x: area.width.saturating_sub(width),
            y: area.height.saturating_sub(3),
            width,
            height: 1,
        };

        // Binary fade: dim after 2s
        let faded = elapsed.as_secs_f32() > 2.0;
        let span_style = Style::default()
            .fg(t.bg())
            .bg(t.accent2())
            .add_modifier(Modifier::BOLD)
            .add_modifier(if faded { Modifier::DIM } else { Modifier::empty() });

        f.render_widget(Clear, toast_area);
        f.render_widget(
            Paragraph::new(Span::styled(format!(" {} ", msg), span_style)),
            toast_area,
        );
    }
}
