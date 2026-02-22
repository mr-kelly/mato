use crate::client::persistence::{SavedDesk, SavedOffice, SavedState, SavedTab};
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
use std::collections::HashMap;
use std::io;

#[derive(Deserialize, Clone)]
#[serde(untagged)]
enum LocalizedText {
    Plain(String),
    ByLang(HashMap<String, String>),
}

impl LocalizedText {
    fn resolve(&self, language: Language) -> &str {
        match self {
            Self::Plain(s) => s.as_str(),
            Self::ByLang(map) => map
                .get(language.code())
                .or_else(|| map.get("en"))
                .map(|s| s.as_str())
                .unwrap_or(""),
        }
    }
}

#[derive(Deserialize, Clone)]
struct TemplateMetadata {
    name: LocalizedText,
    description: LocalizedText,
    details: LocalizedText,
}

#[derive(Deserialize, Clone)]
struct TemplateTab {
    id: String,
    name: LocalizedText,
}

#[derive(Deserialize, Clone)]
struct TemplateDesk {
    id: String,
    name: LocalizedText,
    tabs: Vec<TemplateTab>,
    active_tab: usize,
}

#[derive(Deserialize, Clone)]
struct TemplateOffice {
    id: String,
    name: LocalizedText,
    desks: Vec<TemplateDesk>,
    active_desk: usize,
}

#[derive(Deserialize)]
struct TemplateFile {
    metadata: TemplateMetadata,
    offices: Vec<TemplateOffice>,
    current_office: usize,
}

// Embed templates at compile time
const SOLO_DEVELOPER: &str = include_str!("../../templates/solo-developer.json");
const ONE_PERSON_COMPANY: &str = include_str!("../../templates/one-person-company.json");
const FULLSTACK_DEVELOPER: &str = include_str!("../../templates/fullstack-developer.json");
const DATA_SCIENTIST: &str = include_str!("../../templates/data-scientist.json");
const POWER_USER: &str = include_str!("../../templates/power-user.json");
const MARKETING_OPS: &str = include_str!("../../templates/marketing-ops.json");
const FINANCIAL_TRADER: &str = include_str!("../../templates/financial-trader.json");
const HR_ADMIN: &str = include_str!("../../templates/hr-admin.json");
const START_FROM_SCRATCH: &str = include_str!("../../templates/minimal.json");

const ONBOARDING_ASCII_LOGO: [&str; 5] = [
    "███╗   ███╗ █████╗ ████████╗ ██████╗",
    "████╗ ████║██╔══██╗╚══██╔══╝██╔═══██╗",
    "██╔████╔██║███████║   ██║   ██║   ██║",
    "██║╚██╔╝██║██╔══██║   ██║   ██║   ██║",
    "██║ ╚═╝ ██║██║  ██║   ██║   ╚██████╔╝",
];

struct Template {
    metadata: TemplateMetadata,
    offices: Vec<TemplateOffice>,
    current_office: usize,
}

impl Template {
    fn localized_metadata(&self, language: Language) -> (&str, &str, &str) {
        (
            self.metadata.name.resolve(language),
            self.metadata.description.resolve(language),
            self.metadata.details.resolve(language),
        )
    }
}

#[derive(Copy, Clone)]
enum Language {
    English,
    SimplifiedChinese,
    TraditionalChinese,
    Japanese,
    Korean,
}

impl Language {
    fn previous(self) -> Self {
        match self {
            Self::English => Self::Korean,
            Self::SimplifiedChinese => Self::English,
            Self::TraditionalChinese => Self::SimplifiedChinese,
            Self::Japanese => Self::TraditionalChinese,
            Self::Korean => Self::Japanese,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::English => Self::SimplifiedChinese,
            Self::SimplifiedChinese => Self::TraditionalChinese,
            Self::TraditionalChinese => Self::Japanese,
            Self::Japanese => Self::Korean,
            Self::Korean => Self::English,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::SimplifiedChinese => "简体中文",
            Self::TraditionalChinese => "繁體中文",
            Self::Japanese => "日本語",
            Self::Korean => "한국어",
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::SimplifiedChinese => "zh-CN",
            Self::TraditionalChinese => "zh-TW",
            Self::Japanese => "ja",
            Self::Korean => "ko",
        }
    }
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
        metadata: file.metadata,
        offices: file.offices,
        current_office: file.current_office,
    }
}

fn get_templates() -> Vec<Template> {
    vec![
        parse_template(SOLO_DEVELOPER),
        parse_template(ONE_PERSON_COMPANY),
        parse_template(FULLSTACK_DEVELOPER),
        parse_template(DATA_SCIENTIST),
        parse_template(POWER_USER),
        parse_template(MARKETING_OPS),
        parse_template(FINANCIAL_TRADER),
        parse_template(HR_ADMIN),
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
    let mut language = Language::English;
    let mut office_name = OfficeNameDraft::new(default_office_name());
    let mut result = None;

    loop {
        terminal
            .draw(|f| draw_onboarding(f, &mut list_state, &templates, language, &office_name))?;

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
                    KeyCode::Left => language = language.previous(),
                    KeyCode::Right => language = language.next(),
                    KeyCode::Enter => {
                        let selected = list_state.selected().unwrap_or(0);
                        result = apply_template_return(
                            &templates[selected],
                            language,
                            &office_name.committed,
                        );
                        break;
                    }
                    KeyCode::Char('r') => office_name.start_edit(),
                    KeyCode::Esc | KeyCode::Char('q') => break,
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
    language: Language,
    office_name: &OfficeNameDraft,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    let mut title_lines: Vec<Line> = ONBOARDING_ASCII_LOGO
        .iter()
        .map(|line| {
            Line::from(Span::styled(
                *line,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();
    title_lines.push(Line::from(Span::styled(
        ui_text(language, UiText::WelcomeSubtitle),
        Style::default().fg(Color::Gray),
    )));

    let title = Paragraph::new(title_lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let language_line = Paragraph::new(format!(
        "{}: < {} > ({})",
        ui_text(language, UiText::LanguageLabel),
        language.label(),
        ui_text(language, UiText::LanguageHint)
    ))
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::Gray))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(language_line, chunks[1]);

    let office_line = if office_name.in_edit {
        format!(
            "{}: {}█",
            ui_text(language, UiText::OfficeName),
            office_name.editing
        )
    } else {
        format!(
            "{}: {}  ({})",
            ui_text(language, UiText::OfficeName),
            office_name.committed,
            ui_text(language, UiText::RenameHint)
        )
    };
    let office_style = if office_name.in_edit {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let office_widget = Paragraph::new(office_line)
        .alignment(Alignment::Center)
        .style(office_style)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(office_widget, chunks[2]);

    let items: Vec<ListItem> = templates
        .iter()
        .map(|t| {
            let (name, desc, _) = t.localized_metadata(language);
            ListItem::new(vec![
                Line::from(Span::styled(
                    name,
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(desc, Style::default().fg(Color::DarkGray))),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(
                    " {} ",
                    ui_text(language, UiText::ChooseTemplateTitle)
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 40, 60))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, chunks[3], list_state);

    let selected = list_state.selected().unwrap_or(0);
    let template = &templates[selected];
    let (_, _, details_text) = template.localized_metadata(language);
    let details = Paragraph::new(details_text)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(format!(" {} ", ui_text(language, UiText::DetailsTitle)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(details, chunks[4]);

    let help_text = if office_name.in_edit {
        ui_text(language, UiText::HelpEditing)
    } else {
        ui_text(language, UiText::HelpNormal)
    };
    let help = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[5]);
}

fn apply_template_return(
    template: &Template,
    language: Language,
    office_name: &str,
) -> Option<SavedState> {
    let offices: Vec<SavedOffice> = template
        .offices
        .iter()
        .map(|o| SavedOffice {
            id: o.id.clone(),
            name: o.name.resolve(language).to_string(),
            desks: o
                .desks
                .iter()
                .map(|d| SavedDesk {
                    id: d.id.clone(),
                    name: d.name.resolve(language).to_string(),
                    tabs: d
                        .tabs
                        .iter()
                        .map(|t| SavedTab {
                            id: t.id.clone(),
                            name: t.name.resolve(language).to_string(),
                        })
                        .collect(),
                    active_tab: d.active_tab,
                })
                .collect(),
            active_desk: o.active_desk,
        })
        .collect();

    let mut state = SavedState {
        offices,
        current_office: template.current_office,
    };

    if let Some(first_office) = state.offices.first_mut() {
        first_office.name = office_name.to_string();
    }

    Some(state)
}

#[derive(Copy, Clone)]
enum UiText {
    WelcomeSubtitle,
    LanguageLabel,
    LanguageHint,
    OfficeName,
    RenameHint,
    ChooseTemplateTitle,
    DetailsTitle,
    HelpEditing,
    HelpNormal,
}

fn ui_text(language: Language, key: UiText) -> &'static str {
    match (language, key) {
        (_, UiText::WelcomeSubtitle) => "Multi-Agent Terminal Office",

        (Language::English, UiText::LanguageLabel) => "Language",
        (Language::SimplifiedChinese, UiText::LanguageLabel) => "语言",
        (Language::TraditionalChinese, UiText::LanguageLabel) => "語言",
        (Language::Japanese, UiText::LanguageLabel) => "言語",
        (Language::Korean, UiText::LanguageLabel) => "언어",

        (Language::English, UiText::LanguageHint) => "Left/Right to switch",
        (Language::SimplifiedChinese, UiText::LanguageHint) => "左右切换",
        (Language::TraditionalChinese, UiText::LanguageHint) => "左右切換",
        (Language::Japanese, UiText::LanguageHint) => "左右キーで切替",
        (Language::Korean, UiText::LanguageHint) => "좌우로 전환",

        (Language::English, UiText::OfficeName) => "Office Name",
        (Language::SimplifiedChinese, UiText::OfficeName) => "Office 名称",
        (Language::TraditionalChinese, UiText::OfficeName) => "Office 名稱",
        (Language::Japanese, UiText::OfficeName) => "Office 名",
        (Language::Korean, UiText::OfficeName) => "Office 이름",

        (Language::English, UiText::RenameHint) => "r to rename",
        (Language::SimplifiedChinese, UiText::RenameHint) => "按 r 重命名",
        (Language::TraditionalChinese, UiText::RenameHint) => "按 r 重新命名",
        (Language::Japanese, UiText::RenameHint) => "r で名前変更",
        (Language::Korean, UiText::RenameHint) => "r로 이름 변경",

        (Language::English, UiText::ChooseTemplateTitle) => "Choose an Office Template",
        (Language::SimplifiedChinese, UiText::ChooseTemplateTitle) => "选择 Office 模板",
        (Language::TraditionalChinese, UiText::ChooseTemplateTitle) => "選擇 Office 模板",
        (Language::Japanese, UiText::ChooseTemplateTitle) => "Office テンプレートを選択",
        (Language::Korean, UiText::ChooseTemplateTitle) => "Office 템플릿 선택",

        (Language::English, UiText::DetailsTitle) => "Details",
        (Language::SimplifiedChinese, UiText::DetailsTitle) => "详情",
        (Language::TraditionalChinese, UiText::DetailsTitle) => "詳情",
        (Language::Japanese, UiText::DetailsTitle) => "詳細",
        (Language::Korean, UiText::DetailsTitle) => "상세",

        (Language::English, UiText::HelpEditing) => "Type name  |  Enter Save  |  Esc Cancel",
        (Language::SimplifiedChinese, UiText::HelpEditing) => {
            "输入名称  |  Enter 保存  |  Esc 取消"
        }
        (Language::TraditionalChinese, UiText::HelpEditing) => {
            "輸入名稱  |  Enter 儲存  |  Esc 取消"
        }
        (Language::Japanese, UiText::HelpEditing) => "名前入力  |  Enter 保存  |  Esc キャンセル",
        (Language::Korean, UiText::HelpEditing) => "이름 입력  |  Enter 저장  |  Esc 취소",

        (Language::English, UiText::HelpNormal) => {
            "↑↓ Navigate  |  ←→ Language  |  Enter Start  |  r Rename Office  |  Esc/q Cancel"
        }
        (Language::SimplifiedChinese, UiText::HelpNormal) => {
            "↑↓ 选择  |  ←→ 语言  |  Enter 开始  |  r 重命名  |  Esc/q 取消"
        }
        (Language::TraditionalChinese, UiText::HelpNormal) => {
            "↑↓ 選擇  |  ←→ 語言  |  Enter 開始  |  r 重新命名  |  Esc/q 取消"
        }
        (Language::Japanese, UiText::HelpNormal) => {
            "↑↓ 選択  |  ←→ 言語  |  Enter 開始  |  r 名前変更  |  Esc/q キャンセル"
        }
        (Language::Korean, UiText::HelpNormal) => {
            "↑↓ 선택  |  ←→ 언어  |  Enter 시작  |  r 이름 변경  |  Esc/q 취소"
        }
    }
}

fn default_office_name() -> String {
    fn capitalize_first(s: &str) -> String {
        let mut chars = s.chars();
        let Some(first) = chars.next() else {
            return String::new();
        };
        let mut out = String::new();
        out.extend(first.to_uppercase());
        out.extend(chars);
        out
    }

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
        .and_then(|u| clean_token(&u, 12));

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

    // Choose one identity token only: username first, hostname fallback.
    let base = user.or(host).unwrap_or_else(|| "My".to_string());
    let base = capitalize_first(&base);

    let mut name = format!("{base} AI Office");
    if name.chars().count() > 24 {
        let keep = 24usize.saturating_sub(" AI Office".chars().count());
        let short: String = base.chars().take(keep).collect();
        name = format!("{short} AI Office");
    }
    name
}
