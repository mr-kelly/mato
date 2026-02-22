use crate::client::persistence::SavedState;
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
struct TemplateMetadata {
    name: String,
    description: String,
    details: String,
}

#[derive(Deserialize)]
struct TemplateFile {
    metadata: TemplateMetadata,
}

#[derive(Deserialize, Clone, Default)]
struct TemplateI18n {
    #[serde(default)]
    metadata: HashMap<String, TemplateMetadata>,
    #[serde(default)]
    names: HashMap<String, HashMap<String, String>>,
}

#[derive(Deserialize, Default)]
struct I18nCatalog {
    #[serde(default)]
    templates: HashMap<String, TemplateI18n>,
}

// Embed templates at compile time
const SOLO_DEVELOPER: &str = include_str!("../../templates/solo-developer.json");
const ONE_PERSON_COMPANY: &str = include_str!("../../templates/one-person-company.json");
const FULLSTACK_DEVELOPER: &str = include_str!("../../templates/fullstack-developer.json");
const DATA_SCIENTIST: &str = include_str!("../../templates/data-scientist.json");
const POWER_USER: &str = include_str!("../../templates/power-user.json");
const START_FROM_SCRATCH: &str = include_str!("../../templates/minimal.json");
const TEMPLATE_I18N: &str = include_str!("../../templates/i18n.json");

struct Template {
    metadata: TemplateMetadata,
    content: &'static str,
    i18n: TemplateI18n,
}

impl Template {
    fn localized_metadata(&self, language: Language) -> (&str, &str, &str) {
        if let Some(meta) = self.i18n.metadata.get(language.code()) {
            (&meta.name, &meta.description, &meta.details)
        } else {
            (
                &self.metadata.name,
                &self.metadata.description,
                &self.metadata.details,
            )
        }
    }

    fn localize_state_name(&self, language: Language, name: &str) -> String {
        self.i18n
            .names
            .get(language.code())
            .and_then(|m| m.get(name))
            .cloned()
            .unwrap_or_else(|| name.to_string())
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

fn parse_template(key: &'static str, content: &'static str, catalog: &I18nCatalog) -> Template {
    let file: TemplateFile = serde_json::from_str(content).expect("Failed to parse template");
    Template {
        metadata: file.metadata,
        content,
        i18n: catalog.templates.get(key).cloned().unwrap_or_default(),
    }
}

fn get_templates() -> Vec<Template> {
    let catalog: I18nCatalog = serde_json::from_str(TEMPLATE_I18N).expect("Failed to parse i18n");
    vec![
        parse_template("solo-developer", SOLO_DEVELOPER, &catalog),
        parse_template("one-person-company", ONE_PERSON_COMPANY, &catalog),
        parse_template("fullstack-developer", FULLSTACK_DEVELOPER, &catalog),
        parse_template("data-scientist", DATA_SCIENTIST, &catalog),
        parse_template("power-user", POWER_USER, &catalog),
        parse_template("minimal", START_FROM_SCRATCH, &catalog),
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
                        )?;
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
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            ui_text(language, UiText::WelcomeTitle),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            ui_text(language, UiText::WelcomeSubtitle),
            Style::default().fg(Color::Gray),
        )),
    ])
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
) -> io::Result<Option<SavedState>> {
    let mut state: SavedState = serde_json::from_str(template.content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if !matches!(language, Language::English) {
        for office in &mut state.offices {
            for desk in &mut office.desks {
                desk.name = template.localize_state_name(language, &desk.name);
                for tab in &mut desk.tabs {
                    tab.name = template.localize_state_name(language, &tab.name);
                }
            }
        }
    }

    if let Some(first_office) = state.offices.first_mut() {
        first_office.name = office_name.to_string();
    }
    Ok(Some(state))
}

#[derive(Copy, Clone)]
enum UiText {
    WelcomeTitle,
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
        (Language::English, UiText::WelcomeTitle) => "Welcome to Mato",
        (Language::SimplifiedChinese, UiText::WelcomeTitle) => "欢迎使用 Mato",
        (Language::TraditionalChinese, UiText::WelcomeTitle) => "歡迎使用 Mato",
        (Language::Japanese, UiText::WelcomeTitle) => "Mato へようこそ",
        (Language::Korean, UiText::WelcomeTitle) => "Mato에 오신 것을 환영합니다",

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
