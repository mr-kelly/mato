use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use crate::app::{App, EscMode, Focus, RenameTarget};

pub const BG:         Color = Color::Rgb(18,  18,  28);
pub const SURFACE:    Color = Color::Rgb(28,  28,  42);
pub const BORDER:     Color = Color::Rgb(60,  60,  90);
pub const ACCENT:     Color = Color::Rgb(100, 160, 255);
pub const ACCENT2:    Color = Color::Rgb(80,  220, 160);
pub const FG:         Color = Color::Rgb(210, 210, 230);
pub const FG_DIM:     Color = Color::Rgb(100, 100, 130);
pub const SEL_BG:     Color = Color::Rgb(40,  60,  100);

pub fn border_style(active: bool) -> Style {
    if active { Style::default().fg(ACCENT) } else { Style::default().fg(BORDER) }
}
fn title_style(active: bool) -> Style {
    if active { Style::default().fg(ACCENT).add_modifier(Modifier::BOLD) }
    else { Style::default().fg(FG_DIM) }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    f.render_widget(Block::default().style(Style::default().bg(BG)), f.area());

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
    app.spawn_active_pty(); // ensure active tab always has a PTY

    app.sidebar_area = cols[0];
    app.topbar_area  = main_rows[0];
    app.content_area = main_rows[1];

    draw_sidebar(f, app, cols[0]);
    draw_topbar(f, app, main_rows[0]);
    draw_terminal(f, app, main_rows[1]);
    draw_statusbar(f, app, root[1]);

    // Rename overlay
    if app.rename.is_some() {
        draw_rename_popup(f, app);
    }
}

fn draw_statusbar(f: &mut Frame, app: &App, area: Rect) {
    let keys: &[(&str, &str)] = if app.rename.is_some() {
        &[("Enter", "Confirm"), ("Esc", "Cancel")]
    } else if app.esc_mode == EscMode::Pending {
        &[("← a", "Sidebar"), ("↑ w", "Topbar")]
    } else {
        match app.focus {
            Focus::Sidebar => &[("↑↓", "Navigate"), ("n", "New Task"), ("x", "Close"), ("r", "Rename"), ("q", "Quit")],
            Focus::Topbar  => &[("←→", "Switch Tab"), ("t", "New Tab"), ("w", "Close Tab"), ("r", "Rename Tab"), ("Enter", "Focus")],
            Focus::Content => &[("Esc", "combo"), ("keys→shell", "")],
        }
    };
    let mut spans: Vec<Span> = vec![Span::raw(" ")];
    for (key, desc) in keys {
        spans.push(Span::styled(format!(" {key} "), Style::default().fg(BG).bg(ACCENT).add_modifier(Modifier::BOLD)));
        if !desc.is_empty() {
            spans.push(Span::styled(format!(" {desc}  "), Style::default().fg(FG_DIM)));
        } else {
            spans.push(Span::raw("  "));
        }
    }
    f.render_widget(Paragraph::new(Line::from(spans)).style(Style::default().bg(SURFACE)), area);
}

fn draw_sidebar(f: &mut Frame, app: &mut App, area: Rect) {
    let active = app.focus == Focus::Sidebar;
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let btn_style = if active {
        Style::default().fg(BG).bg(ACCENT).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(FG_DIM).bg(SURFACE)
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled("  ＋  New Task  ", btn_style)]))
            .style(btn_style)
            .block(Block::default().borders(Borders::ALL).border_style(border_style(active)).style(Style::default().bg(SURFACE))),
        rows[0],
    );
    app.new_task_area = rows[0];

    let items: Vec<ListItem> = app.tasks.iter().enumerate().map(|(i, t)| {
        let sel = app.list_state.selected() == Some(i);
        ListItem::new(Line::from(vec![
            Span::styled(if sel { " ▶ " } else { "   " }, Style::default().fg(ACCENT)),
            Span::styled(t.name.clone(), Style::default().fg(if sel { FG } else { FG_DIM })),
        ])).style(Style::default().bg(if sel { SEL_BG } else { SURFACE }))
    }).collect();

    app.sidebar_list_area = rows[1];
    f.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL)
                .title(Span::styled(" Tasks ", title_style(active)))
                .border_style(border_style(active))
                .style(Style::default().bg(SURFACE))),
        rows[1],
        &mut app.list_state,
    );
}

fn draw_topbar(f: &mut Frame, app: &mut App, area: Rect) {
    let active = app.focus == Focus::Topbar;
    f.render_widget(
        Block::default().borders(Borders::ALL).border_style(border_style(active)).style(Style::default().bg(SURFACE)),
        area,
    );

    let task = &app.tasks[app.selected()];
    let inner_w = area.width.saturating_sub(2);
    let inner = Rect { x: area.x + 1, y: area.y + 1, width: inner_w, height: 1 };

    // Ensure active tab is visible: adjust tab_scroll
    let at = task.active_tab;
    if at < app.tab_scroll { app.tab_scroll = at; }
    // measure widths to find if active tab is past the right edge
    let plus_w = 7u16; // "  ＋  " + space
    let tab_widths: Vec<u16> = task.tabs.iter().map(|t| format!("  {}  ", t.name).len() as u16 + 1).collect();
    // scroll forward until active tab fits
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

    // left scroll indicator
    if app.tab_scroll > 0 {
        spans.push(Span::styled("‹ ", Style::default().fg(ACCENT2)));
        x += 2;
    }

    let mut available = inner_w.saturating_sub(plus_w + if app.tab_scroll > 0 { 2 } else { 0 });
    for i in app.tab_scroll..task.tabs.len() {
        let tab = &task.tabs[i];
        let label = format!("  {}  ", tab.name);
        let w = label.len() as u16;
        if w + 1 > available { break; }
        let is_active = i == task.active_tab;
        let style = if is_active {
            Style::default().fg(BG).bg(ACCENT).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(FG_DIM)
        };
        spans.push(Span::styled(label, style));
        spans.push(Span::raw(" "));
        app.tab_areas.push(Rect { x, y: inner.y, width: w, height: 1 });
        x += w + 1;
        available = available.saturating_sub(w + 1);
    }

    // right scroll indicator if more tabs exist
    let last_rendered = app.tab_scroll + app.tab_areas.len();
    if last_rendered < task.tabs.len() {
        spans.push(Span::styled(" ›", Style::default().fg(ACCENT2)));
    }

    // "+" new tab button
    let plus = "  ＋  ";
    spans.push(Span::styled(plus, Style::default().fg(ACCENT2)));
    app.new_tab_area = Rect { x, y: inner.y, width: plus_w, height: 1 };

    f.render_widget(Paragraph::new(Line::from(spans)), inner);
}

fn draw_terminal(f: &mut Frame, app: &App, area: Rect) {
    let active = app.focus == Focus::Content;
    let task = &app.tasks[app.selected()];
    let tab = task.active_tab_ref();

    f.render_widget(
        Block::default().borders(Borders::ALL)
            .title(Span::styled(format!(" {} ", tab.name), title_style(active)))
            .border_style(border_style(active))
            .style(Style::default().bg(Color::Black)),
        area,
    );

    let Some(pty) = &tab.pty else { return };
    let parser = pty.parser.lock().unwrap();
    let screen = parser.screen();
    let (ix, iy) = (area.x + 1, area.y + 1);
    let (iw, ih) = (area.width.saturating_sub(2), area.height.saturating_sub(2));

    for row in 0..ih {
        let spans: Vec<Span> = (0..iw).map(|col| {
            let def = vt100::Cell::default();
            let cell = screen.cell(row, col).unwrap_or(&def);
            let ch = cell.contents().chars().next().unwrap_or(' ');
            let mut style = Style::default();
            if let Some(c) = vt_color(cell.fgcolor()) { style = style.fg(c); }
            if let Some(c) = vt_color(cell.bgcolor()) { style = style.bg(c); }
            if cell.bold()      { style = style.add_modifier(Modifier::BOLD); }
            if cell.italic()    { style = style.add_modifier(Modifier::ITALIC); }
            if cell.underline() { style = style.add_modifier(Modifier::UNDERLINED); }
            Span::styled(ch.to_string(), style)
        }).collect();
        f.render_widget(Paragraph::new(Line::from(spans)), Rect { x: ix, y: iy + row, width: iw, height: 1 });
    }

    let (cr, cc) = screen.cursor_position();
    if cr < ih && cc < iw {
        let def = vt100::Cell::default();
        let cell = screen.cell(cr, cc).unwrap_or(&def);
        let ch = cell.contents().chars().next().unwrap_or(' ');
        f.render_widget(
            Paragraph::new(Span::styled(ch.to_string(), Style::default().bg(ACCENT2).fg(Color::Black))),
            Rect { x: ix + cc, y: iy + cr, width: 1, height: 1 },
        );
    }
}

fn draw_rename_popup(f: &mut Frame, app: &App) {
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
            Span::styled(buf.clone(), Style::default().fg(FG)),
            Span::styled("█", Style::default().fg(ACCENT)),
        ]))
        .block(Block::default().borders(Borders::ALL)
            .title(Span::styled(label, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
            .border_style(Style::default().fg(ACCENT))
            .style(Style::default().bg(SURFACE))),
        popup,
    );
}

fn vt_color(c: vt100::Color) -> Option<Color> {
    match c {
        vt100::Color::Rgb(r, g, b) => Some(Color::Rgb(r, g, b)),
        vt100::Color::Idx(i)       => Some(Color::Indexed(i)),
        vt100::Color::Default      => None,
    }
}
