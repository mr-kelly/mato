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

#[derive(Clone)]
struct OfficeNameDraft {
    committed: String,
    editing: String,
    in_edit: bool,
}

impl OfficeNameDraft {
    fn new(default_name: String) -> Self {
        Self {
            committed: default_name.clone(),
            editing: default_name,
            in_edit: false,
        }
    }

    fn start_edit(&mut self) {
        self.editing = self.committed.clone();
        self.in_edit = true;
    }

    fn cancel(&mut self) {
        self.editing = self.committed.clone();
        self.in_edit = false;
    }

    fn commit(&mut self) {
        let trimmed = self.editing.trim();
        if !trimmed.is_empty() {
            self.committed = trimmed.to_string();
        }
        self.in_edit = false;
    }
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
    let mut office_name = OfficeNameDraft::new(default_office_name());
    let mut result = None;

    loop {
        terminal.draw(|f| draw_onboarding(f, &mut list_state, &templates, &office_name))?;

        if let Event::Key(key) = event::read()? {
            if office_name.in_edit {
                match key.code {
                    KeyCode::Enter => office_name.commit(),
                    KeyCode::Esc => office_name.cancel(),
                    KeyCode::Backspace => {
                        office_name.editing.pop();
                    }
                    KeyCode::Char(c) => {
                        if !c.is_control() {
                            office_name.editing.push(c);
                        }
                    }
                    _ => {}
                }
            } else {
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
                        result = apply_template_return(&templates[selected], &office_name.committed)?;
                        break;
                    }
                    KeyCode::Char('r') => {
                        office_name.start_edit();
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(result)
}

fn draw_onboarding(
    f: &mut Frame,
    list_state: &mut ListState,
    templates: &[Template],
    office_name: &OfficeNameDraft,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            "Welcome to Mato",
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

    let office_line = if office_name.in_edit {
        format!("Office Name: {}█", office_name.editing)
    } else {
        format!("Office Name: {}  (r to rename)", office_name.committed)
    };
    let office_style = if office_name.in_edit {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let office_widget = Paragraph::new(office_line)
        .alignment(Alignment::Center)
        .style(office_style)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(office_widget, chunks[1]);

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
    f.render_stateful_widget(list, chunks[2], list_state);

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
    f.render_widget(details, chunks[3]);

    let help_text = if office_name.in_edit {
        "Type name  |  Enter Save  |  Esc Cancel"
    } else {
        "↑↓ Navigate  |  Enter Start  |  r Rename Office  |  Esc/q Minimal"
    };
    let help = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[4]);
}

fn apply_template_return(template: &Template, office_name: &str) -> io::Result<Option<SavedState>> {
    let mut state: SavedState = serde_json::from_str(template.content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if let Some(first_office) = state.offices.first_mut() {
        first_office.name = office_name.to_string();
    }
    Ok(Some(state))
}

fn default_office_name() -> String {
    fn clean_token(s: &str, max_len: usize) -> Option<String> {
        let filtered: String = s
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        let token = filtered.trim_matches(|c: char| c == '-' || c == '_');
        if token.is_empty() {
            None
        } else {
            Some(token.chars().take(max_len).collect())
        }
    }

    let user = std::env::var("USER")
        .ok()
        .or_else(|| std::env::var("USERNAME").ok())
        .and_then(|u| clean_token(&u, 10));

    let host_env = std::env::var("HOSTNAME")
        .ok()
        .or_else(|| std::env::var("COMPUTERNAME").ok())
        .or_else(|| std::fs::read_to_string("/etc/hostname").ok())
        .unwrap_or_default();
    let host_short = host_env
        .split('.')
        .next()
        .unwrap_or(host_env.as_str())
        .trim()
        .to_string();
    let host = clean_token(&host_short, 12);

    let base = match (user, host) {
        (Some(u), Some(h)) => {
            if u.eq_ignore_ascii_case(&h) {
                h
            } else {
                format!("{u}@{h}")
            }
        }
        (Some(u), None) => u,
        (None, Some(h)) => h,
        (None, None) => "My".to_string(),
    };

    let mut name = format!("{base} Office");
    if name.chars().count() > 24 {
        let keep = 24usize.saturating_sub(" Office".chars().count());
        let short: String = base.chars().take(keep).collect();
        name = format!("{short} Office");
    }
    name
}
