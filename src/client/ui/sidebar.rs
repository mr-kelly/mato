use crate::client::app::{App, Focus};
use crate::theme::ThemeColors;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub(super) fn draw_sidebar(f: &mut Frame, app: &mut App, area: Rect, t: &ThemeColors) {
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
            Style::default().add_modifier(ratatui::style::Modifier::BOLD | ratatui::style::Modifier::REVERSED)
        } else {
            Style::default()
                .fg(t.fg())
                .bg(t.surface())
                .add_modifier(ratatui::style::Modifier::BOLD)
        }
    } else if t.follow_terminal {
        Style::default().add_modifier(ratatui::style::Modifier::DIM)
    } else {
        Style::default().fg(t.fg_dim()).bg(t.surface())
    };
    f.render_widget(
        ratatui::widgets::Paragraph::new(Line::from(vec![Span::styled(office_text, office_style)]))
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(super::border_type(t, active))
                    .border_style(super::border_style(t, active))
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
                    Style::default().add_modifier(ratatui::style::Modifier::BOLD | ratatui::style::Modifier::REVERSED)
                } else {
                    Style::default().add_modifier(ratatui::style::Modifier::DIM)
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
                .border_type(super::border_type(t, active))
                .title(Span::styled(" Desks ", super::title_style(t, active)))
                .border_style(super::border_style(t, active))
                .style(Style::default().bg(t.surface())),
        ),
        rows[1],
        &mut app.list_state,
    );
}
