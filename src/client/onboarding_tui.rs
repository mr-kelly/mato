use std::io;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use serde::Deserialize;
use crate::client::persistence::SavedState;

#[derive(Deserialize)]
struct TemplateMetadata {
    name: String,
    description: String,
    details: String,
}

#[derive(Deserialize)]
struct TemplateFile {
    metadata: TemplateMetadata,
}

// Embed templates at compile time
const SOLO_DEVELOPER: &str = include_str!("../../templates/solo-developer.json");
const ONE_PERSON_COMPANY: &str = include_str!("../../templates/one-person-company.json");
const FULLSTACK_DEVELOPER: &str = include_str!("../../templates/fullstack-developer.json");
const DATA_SCIENTIST: &str = include_str!("../../templates/data-scientist.json");
const POWER_USER: &str = include_str!("../../templates/power-user.json");
const START_FROM_SCRATCH: &str = include_str!("../../templates/minimal.json");

struct Template {
    name: String,
    description: String,
    details: String,
    content: &'static str,
}

fn parse_template(content: &'static str) -> Template {
    let file: TemplateFile = serde_json::from_str(content).expect("Failed to parse template");
    Template {
        name: file.metadata.name,
        description: file.metadata.description,
        details: file.metadata.details,
        content,
    }
}

fn get_templates() -> Vec<Template> {
    vec![
        parse_template(SOLO_DEVELOPER),
        parse_template(ONE_PERSON_COMPANY),
        parse_template(FULLSTACK_DEVELOPER),
        parse_template(DATA_SCIENTIST),
        parse_template(POWER_USER),
        parse_template(START_FROM_SCRATCH),
    ]
}

pub fn show_onboarding_tui() -> io::Result<Option<SavedState>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let templates = get_templates();
    let mut list_state = ListState::default();
    list_state.select(Some(0));
    let mut result = None;

    loop {
        terminal.draw(|f| draw_onboarding(f, &mut list_state, &templates))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    let selected = list_state.selected().unwrap_or(0);
                    if selected > 0 {
                        list_state.select(Some(selected - 1));
                    }
                }
                KeyCode::Down => {
                    let selected = list_state.selected().unwrap_or(0);
                    if selected < templates.len() - 1 {
                        list_state.select(Some(selected + 1));
                    }
                }
                KeyCode::Enter => {
                    let selected = list_state.selected().unwrap_or(0);
                    result = apply_template_return(&templates[selected])?;
                    break;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    break;
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(result)
}

fn draw_onboarding(f: &mut Frame, list_state: &mut ListState, templates: &[Template]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            "🎉 Welcome to Mato! 🎉",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Multi-Agent Terminal Office",
            Style::default().fg(Color::Gray),
        )),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Template list
    let items: Vec<ListItem> = templates
        .iter()
        .map(|t| {
            ListItem::new(vec![
                Line::from(Span::styled(&t.name, Style::default().add_modifier(Modifier::BOLD))),
                Line::from(Span::styled(
                    &t.description,
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Choose an Office Template ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 40, 60))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, chunks[1], list_state);

    // Details
    let selected = list_state.selected().unwrap_or(0);
    let template = &templates[selected];
    let details = Paragraph::new(template.details.as_str())
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Details ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(details, chunks[2]);

    // Help
    let help = Paragraph::new("↑↓ Navigate  │  Enter Select  │  Esc/q Minimal")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[3]);
}

fn apply_template_return(template: &Template) -> io::Result<Option<SavedState>> {
    let state: SavedState = serde_json::from_str(template.content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(Some(state))
}
