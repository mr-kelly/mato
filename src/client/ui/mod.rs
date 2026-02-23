use crate::client::app::{App, JumpMode};
use crate::theme::ThemeColors;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, BorderType},
    Frame,
};
use unicode_width::UnicodeWidthStr;

mod overlay;
mod sidebar;
mod terminal;
mod topbar;

pub(super) fn border_style(t: &ThemeColors, active: bool) -> Style {
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

pub(super) fn title_style(t: &ThemeColors, active: bool) -> Style {
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

pub(super) fn border_type(t: &ThemeColors, active: bool) -> BorderType {
    if t.follow_terminal && active {
        BorderType::Thick
    } else {
        BorderType::Plain
    }
}

pub(super) fn display_width(text: &str) -> u16 {
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
        app.sidebar_area = ratatui::layout::Rect::default();
        app.topbar_area = ratatui::layout::Rect::default();
        app.sidebar_list_area = ratatui::layout::Rect::default();
        app.tab_areas.clear();
        app.tab_area_tab_indices.clear();
        app.new_desk_area = ratatui::layout::Rect::default();
        app.new_tab_area = ratatui::layout::Rect::default();
        app.content_area = root[0];

        let tr = root[0].height;
        let tc = root[0].width;
        app.term_rows = tr;
        app.term_cols = tc;
        terminal::draw_terminal(f, app, root[0], &t);
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

        sidebar::draw_sidebar(f, app, cols[0], &t);
        topbar::draw_topbar(f, app, main_rows[0], &t);
        terminal::draw_terminal(f, app, main_rows[1], &t);
    }
    topbar::draw_statusbar(f, app, root[1], &t);

    if let JumpMode::Active = app.jump_mode {
        overlay::draw_jump_mode(f, app, &t);
    }
    if app.show_settings {
        overlay::draw_settings(f, app, &t);
    }
    if app.office_selector.active {
        overlay::draw_office_selector(f, app, &t);
    }
    if app.office_delete_confirm.is_some() {
        overlay::draw_office_delete_confirm(f, app, &t);
    }
    if app.desk_delete_confirm.is_some() {
        overlay::draw_desk_delete_confirm(f, app, &t);
    }
    // Keep rename popup on top of all overlays so it is always visible.
    if app.rename.is_some() {
        overlay::draw_rename_popup(f, app, &t);
    }
}
