use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use crate::client::app::{App, Focus, RenameTarget, JumpMode};
use crate::theme::{ThemeColors, BUILTIN_THEMES};

fn border_style(t: &ThemeColors, active: bool) -> Style {
    if active { Style::default().fg(t.accent()) } else { Style::default().fg(t.border()) }
}
fn title_style(t: &ThemeColors, active: bool) -> Style {
    if active { Style::default().fg(t.accent()).add_modifier(Modifier::BOLD) }
    else { Style::default().fg(t.fg_dim()) }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let t = app.theme.clone();
    f.render_widget(Block::default().style(Style::default().bg(t.bg())), f.area());

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

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
    app.resize_all_ptys(tr, tc);
    app.spawn_active_pty();

    app.sidebar_area = cols[0];
    app.topbar_area  = main_rows[0];
    app.content_area = main_rows[1];

    draw_sidebar(f, app, cols[0], &t);
    draw_topbar(f, app, main_rows[0], &t);
    draw_terminal(f, app, main_rows[1], &t);
    draw_statusbar(f, app, root[1], &t);

    if app.rename.is_some() {
        draw_rename_popup(f, app, &t);
    }
    if let JumpMode::Active = app.jump_mode {
        draw_jump_mode(f, app, &t);
    }
    if app.show_settings {
        draw_settings(f, app, &t);
    }
}

fn draw_statusbar(f: &mut Frame, app: &App, area: Rect, t: &ThemeColors) {
    let keys: &[(&str, &str)] = if app.rename.is_some() {
        &[("Enter", "Confirm"), ("Esc", "Cancel")]
    } else if let JumpMode::Active = app.jump_mode {
        // In Jump Mode, show q for Content and Topbar
        match app.focus {
            Focus::Content | Focus::Topbar => &[("a-z", "Jump"), ("←↑", "Focus"), ("q", "Quit"), ("Esc", "Cancel")],
            Focus::Sidebar => &[("a-z", "Jump"), ("←↑", "Focus"), ("Esc", "Cancel")],
        }
    } else {
        match app.focus {
            Focus::Sidebar => &[("↑↓", "Navigate"), ("n", "New Task"), ("x", "Close"), ("r", "Rename"), ("s", "Settings"), ("q", "Quit")],
            Focus::Topbar  => &[("←→", "Switch Tab"), ("n", "New Tab"), ("x", "Close Tab"), ("r", "Rename"), ("Enter", "Focus"), ("q", "Quit")],
            Focus::Content => &[("Esc", "Jump"), ("keys→shell", "")],
        }
    };
    let mut spans: Vec<Span> = vec![Span::raw(" ")];
    for (key, desc) in keys {
        spans.push(Span::styled(format!(" {key} "), Style::default().fg(t.bg()).bg(t.accent()).add_modifier(Modifier::BOLD)));
        if !desc.is_empty() {
            spans.push(Span::styled(format!(" {desc}  "), Style::default().fg(t.fg_dim())));
        } else {
            spans.push(Span::raw("  "));
        }
    }
    f.render_widget(Paragraph::new(Line::from(spans)).style(Style::default().bg(t.surface())), area);

    // Update available notice on the right
    if let Some(ref ver) = app.update_available {
        let notice = format!(" ↑ Update available: {} — mato.sh ", ver);
        let w = notice.len() as u16;
        if w < area.width {
            f.render_widget(
                Paragraph::new(Span::styled(notice, Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Rect { x: area.x + area.width - w, y: area.y, width: w, height: 1 },
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

    let btn_style = if active {
        Style::default().fg(t.bg()).bg(t.accent()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(t.fg_dim()).bg(t.surface())
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled("  ＋  New Task  ", btn_style)]))
            .style(btn_style)
            .block(Block::default().borders(Borders::ALL).border_style(border_style(t, active)).style(Style::default().bg(t.surface()))),
        rows[0],
    );
    app.new_task_area = rows[0];

    let items: Vec<ListItem> = app.tasks.iter().enumerate().map(|(i, task)| {
        let sel = app.list_state.selected() == Some(i);
        
        // Only show spinner if: has active tabs AND at least one is NOT the current tab
        let has_active_other_tabs = task.tabs.iter().enumerate().any(|(tab_idx, tab)| {
            app.active_tabs.contains(&tab.id) && tab_idx != task.active_tab
        });
        
        let name = if has_active_other_tabs {
            format!("{} {}", task.name, app.get_spinner())
        } else {
            task.name.clone()
        };
        let fg_color = if sel { t.fg() } else { t.fg_dim() };
        ListItem::new(Line::from(vec![
            Span::styled(if sel { " ▶ " } else { "   " }, Style::default().fg(t.accent())),
            Span::styled(name, Style::default().fg(fg_color)),
        ])).style(Style::default().bg(if sel { t.sel_bg() } else { t.surface() }))
    }).collect();

    app.sidebar_list_area = rows[1];
    f.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL)
                .title(Span::styled(" Tasks ", title_style(t, active)))
                .border_style(border_style(t, active))
                .style(Style::default().bg(t.surface()))),
        rows[1],
        &mut app.list_state,
    );
}

fn draw_topbar(f: &mut Frame, app: &mut App, area: Rect, t: &ThemeColors) {
    let active = app.focus == Focus::Topbar;
    f.render_widget(
        Block::default().borders(Borders::ALL).border_style(border_style(t, active)).style(Style::default().bg(t.surface())),
        area,
    );

    let task = &app.tasks[app.selected()];
    let inner_w = area.width.saturating_sub(2);
    let inner = Rect { x: area.x + 1, y: area.y + 1, width: inner_w, height: 1 };

    let at = task.active_tab;
    if at < app.tab_scroll { app.tab_scroll = at; }
    let plus_w = 7u16;
    let tab_widths: Vec<u16> = task.tabs.iter().map(|tb| format!("  {}  ", tb.name).len() as u16 + 1).collect();
    loop {
        let mut used = plus_w;
        let mut last_visible = app.tab_scroll;
        for i in app.tab_scroll..task.tabs.len() {
            if used + tab_widths[i] > inner_w { break; }
            used += tab_widths[i];
            last_visible = i;
        }
        if at <= last_visible || app.tab_scroll >= at { break; }
        app.tab_scroll += 1;
    }

    app.tab_areas.clear();
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
        let w = label.len() as u16;
        if w + 1 > available { break; }
        let style = if is_current_tab {
            Style::default().fg(t.bg()).bg(t.accent()).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.fg_dim()).bg(t.surface())
        };
        spans.push(Span::styled(label, style));
        spans.push(Span::raw(" "));
        app.tab_areas.push(Rect { x, y: inner.y, width: w, height: 1 });
        x += w + 1;
        available = available.saturating_sub(w + 1);
    }

    let last_rendered = app.tab_scroll + app.tab_areas.len();
    if last_rendered < task.tabs.len() {
        spans.push(Span::styled(" ›", Style::default().fg(t.accent2())));
    }

    let plus = "  ＋  ";
    spans.push(Span::styled(plus, Style::default().fg(t.accent2())));
    app.new_tab_area = Rect { x, y: inner.y, width: plus_w, height: 1 };

    let daemon_status = " ⚡ ";
    let status_w = daemon_status.len() as u16;
    let status_x = inner.x + inner_w - status_w;
    if status_x > x + plus_w {
        f.render_widget(
            Paragraph::new(Span::styled(daemon_status, Style::default().fg(t.accent2()))),
            Rect { x: status_x, y: inner.y, width: status_w, height: 1 }
        );
    }

    f.render_widget(Paragraph::new(Line::from(spans)), inner);
}

fn draw_terminal(f: &mut Frame, app: &App, area: Rect, t: &ThemeColors) {
    let active = app.focus == Focus::Content;
    let task = &app.tasks[app.selected()];
    let tab = task.active_tab_ref();

    f.render_widget(
        Block::default().borders(Borders::ALL)
            .title(Span::styled(format!(" {} ", tab.name), title_style(t, active)))
            .border_style(border_style(t, active))
            .style(Style::default().bg(Color::Black)),
        area,
    );

    let (ix, iy) = (area.x + 1, area.y + 1);
    let (iw, ih) = (area.width.saturating_sub(2), area.height.saturating_sub(2));
    let screen = tab.provider.get_screen(ih, iw);

    for (row_idx, line) in screen.lines.iter().enumerate() {
        let spans: Vec<Span> = line.cells.iter().map(|cell| {
            let mut style = Style::default();
            if let Some(fg) = cell.fg { style = style.fg(fg); }
            if let Some(bg) = cell.bg { style = style.bg(bg); }
            if cell.bold      { style = style.add_modifier(Modifier::BOLD); }
            if cell.italic    { style = style.add_modifier(Modifier::ITALIC); }
            if cell.underline { style = style.add_modifier(Modifier::UNDERLINED); }
            Span::styled(cell.ch.to_string(), style)
        }).collect();
        f.render_widget(Paragraph::new(Line::from(spans)), Rect { x: ix, y: iy + row_idx as u16, width: iw, height: 1 });
    }

    let (cr, cc) = screen.cursor;
    if cr < ih && cc < iw {
        f.set_cursor_position((ix + cc, iy + cr));
    }
}

fn draw_jump_mode(f: &mut Frame, app: &App, t: &ThemeColors) {
    let labels = "abcdefghijklmnopqrstuvwxyz";
    let mut label_idx = 0;
    let jump_fg = Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD);

    for (i, _task) in app.tasks.iter().enumerate() {
        if label_idx >= labels.len() { break; }
        let label = labels.chars().nth(label_idx).unwrap();
        label_idx += 1;
        let y = app.sidebar_list_area.y + i as u16;
        let x = app.sidebar_list_area.x + 1;
        if y < app.sidebar_list_area.y + app.sidebar_list_area.height {
            f.render_widget(
                Paragraph::new(Span::styled(format!("[{}]", label), jump_fg)),
                Rect { x, y, width: 3, height: 1 },
            );
        }
    }

    let task_idx = app.selected();
    for (tab_idx, _) in app.tasks[task_idx].tabs.iter().enumerate() {
        if label_idx >= labels.len() { break; }
        if tab_idx >= app.tab_areas.len() { break; }
        let label = labels.chars().nth(label_idx).unwrap();
        label_idx += 1;
        let tab_area = app.tab_areas[tab_idx];
        f.render_widget(
            Paragraph::new(Span::styled(format!("[{}]", label), jump_fg)),
            Rect { x: tab_area.x + 1, y: tab_area.y, width: 3, height: 1 },
        );
    }

    let help_area = Rect {
        x: app.content_area.x + 2,
        y: app.content_area.y + 2,
        width: 50, height: 4,
    };
    
    // Help text varies by focus
    let help_line_3 = match app.focus {
        Focus::Content => " ← ↑ to switch focus | q to quit | ESC to cancel ",
        Focus::Topbar => " ← ↑ to switch focus | q to quit | ESC to cancel ",
        Focus::Sidebar => " ← ↑ to switch focus | ESC to cancel ",
    };
    
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(" JUMP MODE ", jump_fg)),
            Line::from(Span::styled(" Press letter to jump to task/tab ", Style::default().fg(t.fg()))),
            Line::from(Span::styled(help_line_3, Style::default().fg(t.fg_dim()))),
        ])
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)).style(Style::default().bg(t.surface()))),
        help_area,
    );
}

fn draw_rename_popup(f: &mut Frame, app: &App, t: &ThemeColors) {
    let Some((target, buf)) = &app.rename else { return };
    let label = match target {
        RenameTarget::Task(_)    => " Rename Task ",
        RenameTarget::Tab(_, _)  => " Rename Tab ",
    };
    let area = f.area();
    let w = 40u16.min(area.width);
    let popup = Rect {
        x: (area.width.saturating_sub(w)) / 2,
        y: area.height / 2 - 2,
        width: w, height: 3,
    };
    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(buf.clone(), Style::default().fg(t.fg())),
            Span::styled("█", Style::default().fg(t.accent())),
        ]))
        .block(Block::default().borders(Borders::ALL)
            .title(Span::styled(label, Style::default().fg(t.accent()).add_modifier(Modifier::BOLD)))
            .border_style(Style::default().fg(t.accent()))
            .style(Style::default().bg(t.surface()))),
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
        width: w, height: h,
    };
    f.render_widget(Clear, popup);

    let items: Vec<ListItem> = BUILTIN_THEMES.iter().enumerate().map(|(i, name)| {
        let sel = app.settings_selected == i;
        ListItem::new(Line::from(vec![
            Span::styled(if sel { " ▶ " } else { "   " }, Style::default().fg(t.accent())),
            Span::styled(*name, Style::default().fg(if sel { t.fg() } else { t.fg_dim() })),
        ])).style(Style::default().bg(if sel { t.sel_bg() } else { t.surface() }))
    }).collect();

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(app.settings_selected));

    f.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL)
                .title(Span::styled(" Settings — Theme ", Style::default().fg(t.accent()).add_modifier(Modifier::BOLD)))
                .border_style(Style::default().fg(t.accent()))
                .style(Style::default().bg(t.surface())))
            .highlight_style(Style::default().bg(t.sel_bg())),
        popup,
        &mut list_state,
    );
}

