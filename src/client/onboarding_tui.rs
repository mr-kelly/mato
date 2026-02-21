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

// Embed templates at compile time
const POWER_USER: &str = include_str!("../../templates/power-user.json");
const SOLO_DEVELOPER: &str = include_str!("../../templates/solo-developer.json");
const ONE_PERSON_COMPANY: &str = include_str!("../../templates/one-person-company.json");
const FULLSTACK_DEVELOPER: &str = include_str!("../../templates/fullstack-developer.json");
const DATA_SCIENTIST: &str = include_str!("../../templates/data-scientist.json");
const MINIMAL: &str = include_str!("../../templates/minimal.json");

struct Template {
    name: &'static str,
    description: &'static str,
    details: &'static str,
    content: &'static str,
}

const TEMPLATES: &[Template] = &[
    Template {
        name: "⭐ Power User",
        description: "45 tasks, 250+ tabs",
        details: "Complete setup with all AI tools (Claude, Gemini, Codex, Copilot, Cursor, Aider, Continue, Cline, Windsurf, Bolt) and comprehensive business functions. Perfect for serious professionals managing complex workflows.",
        content: POWER_USER,
    },
    Template {
        name: "💻 Solo Developer",
        description: "3 tasks, 8 tabs",
        details: "Focused workspace for individual developers. Includes Development (Editor, Dev Server, Logs), Git & Deploy, and Tools (Database, Docker, Monitor).",
        content: SOLO_DEVELOPER,
    },
    Template {
        name: "💼 One-Person Company",
        description: "4 tasks, 13 tabs",
        details: "Organized by business departments: Engineering, Product, Marketing, and Operations. Perfect for solo entrepreneurs managing multiple business functions.",
        content: ONE_PERSON_COMPANY,
    },
    Template {
        name: "🚀 Full-Stack Developer",
        description: "4 tasks, 11 tabs",
        details: "Multiple projects workspace with Main Project, Side Project, DevOps (Docker, K8s, CI/CD), and Learning sections.",
        content: FULLSTACK_DEVELOPER,
    },
    Template {
        name: "📊 Data Scientist",
        description: "4 tasks, 11 tabs",
        details: "Data-focused workspace with Data Analysis (Jupyter, Python, Viz), ML Training (TensorBoard, GPU), Data Pipeline (ETL, Airflow), and Deployment.",
        content: DATA_SCIENTIST,
    },
    Template {
        name: "✨ Minimal",
        description: "1 task, 1 tab",
        details: "Start from scratch with a clean slate. Perfect if you want to build your own workspace from the ground up.",
        content: MINIMAL,
    },
];

pub fn show_onboarding_tui() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        terminal.draw(|f| draw_onboarding(f, &mut list_state))?;

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
                    if selected < TEMPLATES.len() - 1 {
                        list_state.select(Some(selected + 1));
                    }
                }
                KeyCode::Enter => {
                    let selected = list_state.selected().unwrap_or(0);
                    apply_template(selected)?;
                    break;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    apply_template(5)?; // Default to minimal
                    break;
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn draw_onboarding(f: &mut Frame, list_state: &mut ListState) {
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
    let items: Vec<ListItem> = TEMPLATES
        .iter()
        .map(|t| {
            ListItem::new(vec![
                Line::from(Span::styled(t.name, Style::default().add_modifier(Modifier::BOLD))),
                Line::from(Span::styled(
                    t.description,
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Choose a Workspace Template ")
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
    let template = &TEMPLATES[selected];
    let details = Paragraph::new(template.details)
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

fn apply_template(index: usize) -> io::Result<()> {
    let template = &TEMPLATES[index];
    let state_path = crate::utils::get_state_file_path();
    
    if let Some(parent) = state_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    std::fs::write(&state_path, template.content)?;
    Ok(())
}
