use ratatui::{layout::Rect, widgets::ListState};
use std::collections::HashSet;
use crate::{utils::new_id, client::persistence::load_state, terminal_provider::TerminalProvider, providers::DaemonProvider};
use crate::protocol::{ClientMsg, ServerMsg};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Focus { Sidebar, Topbar, Content }

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum EscMode { None, Pending }

#[derive(PartialEq, Clone)]
pub enum RenameTarget { Task(usize), Tab(usize, usize) }

pub struct TabEntry {
    pub id: String,
    pub name: String,
    pub provider: Box<dyn TerminalProvider>,
}

impl TabEntry {
    pub fn new(name: impl Into<String>) -> Self {
        let id = new_id();
        let socket_path = crate::utils::get_socket_path().to_string_lossy().to_string();
        Self { 
            id: id.clone(), 
            name: name.into(), 
            provider: Box::new(DaemonProvider::new(id, socket_path)) 
        }
    }
    pub fn with_id(id: String, name: impl Into<String>) -> Self {
        let socket_path = crate::utils::get_socket_path().to_string_lossy().to_string();
        Self { 
            id: id.clone(), 
            name: name.into(), 
            provider: Box::new(DaemonProvider::new(id, socket_path)) 
        }
    }

    pub fn spawn_pty(&mut self, rows: u16, cols: u16) {
        self.provider.spawn(rows, cols);
    }

    pub fn resize_pty(&mut self, rows: u16, cols: u16) {
        self.provider.resize(rows, cols);
    }

    pub fn pty_write(&mut self, bytes: &[u8]) {
        self.provider.write(bytes);
    }
}

pub struct Task {
    pub id: String,
    pub name: String,
    pub tabs: Vec<TabEntry>,
    pub active_tab: usize,
}

impl Task {
    pub fn new(name: impl Into<String>) -> Self {
        Self { id: new_id(), name: name.into(), tabs: vec![TabEntry::new("Terminal 1")], active_tab: 0 }
    }

    pub fn active_tab_ref(&self) -> &TabEntry { &self.tabs[self.active_tab] }

    pub fn new_tab(&mut self) {
        let n = self.tabs.len() + 1;
        self.tabs.push(TabEntry::new(format!("Terminal {n}")));
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self) {
        if self.tabs.len() <= 1 { return; }
        self.tabs.remove(self.active_tab);
        self.active_tab = self.active_tab.min(self.tabs.len() - 1);
    }

    pub fn resize_all_ptys(&mut self, rows: u16, cols: u16) {
        for tab in &mut self.tabs { tab.resize_pty(rows, cols); }
    }
}

pub struct App {
    pub tasks: Vec<Task>,
    pub list_state: ListState,
    pub focus: Focus,
    pub esc_mode: EscMode,
    pub rename: Option<(RenameTarget, String)>,
    pub term_rows: u16,
    pub term_cols: u16,
    pub dirty: bool,
    // layout rects
    pub sidebar_list_area: Rect,
    pub sidebar_area: Rect,
    pub topbar_area: Rect,
    pub content_area: Rect,
    pub new_task_area: Rect,
    pub tab_areas: Vec<Rect>,
    pub new_tab_area: Rect,
    pub tab_scroll: usize,
    pub last_click: Option<(u16, u16, std::time::Instant)>,
    /// tab_ids that have been idle for > IDLE_THRESHOLD_SECS
    pub idle_tabs: HashSet<String>,
}

impl App {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        let (tasks, active_task) = if let Ok(s) = load_state() {
            let tasks = s.tasks.into_iter().map(|t| {
                let tabs = t.tabs.into_iter().map(|tb| TabEntry::with_id(tb.id, tb.name)).collect();
                Task { id: t.id, name: t.name, tabs, active_tab: t.active_tab }
            }).collect();
            (tasks, s.active_task)
        } else {
            (vec![Task::new("Task 1")], 0)
        };
        list_state.select(Some(active_task));
        Self {
            tasks, list_state,
            focus: Focus::Sidebar, esc_mode: EscMode::None, rename: None,
            term_rows: 24, term_cols: 80, dirty: false,
            sidebar_list_area: Rect::default(), sidebar_area: Rect::default(),
            topbar_area: Rect::default(), content_area: Rect::default(),
            new_task_area: Rect::default(), tab_areas: vec![], new_tab_area: Rect::default(),
            tab_scroll: 0, last_click: None,
            idle_tabs: HashSet::new(),
        }
    }

    pub fn selected(&self) -> usize { self.list_state.selected().unwrap_or(0) }

    pub fn active_task(&self) -> usize { self.selected() }

    pub fn cur_task_mut(&mut self) -> &mut Task { let i = self.selected(); &mut self.tasks[i] }

    pub fn new_task(&mut self) {
        let n = self.tasks.len() + 1;
        self.tasks.push(Task::new(format!("Task {n}")));
        self.list_state.select(Some(self.tasks.len() - 1));
        self.dirty = true;
    }

    pub fn close_task(&mut self) {
        if self.tasks.len() <= 1 { return; }
        let idx = self.selected();
        self.tasks.remove(idx);
        self.list_state.select(Some(idx.min(self.tasks.len() - 1)));
        self.dirty = true;
    }

    pub fn nav(&mut self, delta: i32) {
        let max = self.tasks.len().saturating_sub(1) as i32;
        let next = (self.selected() as i32 + delta).clamp(0, max) as usize;
        self.list_state.select(Some(next));
        self.tab_scroll = 0;
    }

    pub fn spawn_active_pty(&mut self) {
        let (rows, cols) = (self.term_rows, self.term_cols);
        let i = self.selected();
        let at = self.tasks[i].active_tab;
        self.tasks[i].tabs[at].spawn_pty(rows, cols);
    }

    pub fn resize_all_ptys(&mut self, rows: u16, cols: u16) {
        if self.term_rows == rows && self.term_cols == cols { return; }
        self.term_rows = rows; self.term_cols = cols;
        for task in &mut self.tasks { task.resize_all_ptys(rows, cols); }
    }

    pub fn pty_write(&mut self, bytes: &[u8]) {
        let i = self.selected();
        let at = self.tasks[i].active_tab;
        self.tasks[i].tabs[at].pty_write(bytes);
    }

    /// Start renaming task at sidebar index
    pub fn begin_rename_task(&mut self, idx: usize) {
        let name = self.tasks[idx].name.clone();
        self.rename = Some((RenameTarget::Task(idx), name));
    }

    /// Start renaming active tab of current task
    pub fn begin_rename_tab(&mut self) {
        let ti = self.selected();
        let at = self.tasks[ti].active_tab;
        let name = self.tasks[ti].tabs[at].name.clone();
        self.rename = Some((RenameTarget::Tab(ti, at), name));
    }

    pub fn commit_rename(&mut self) {
        if let Some((target, buf)) = self.rename.take() {
            let name = buf.trim().to_string();
            if name.is_empty() { return; }
            match target {
                RenameTarget::Task(i) => self.tasks[i].name = name,
                RenameTarget::Tab(ti, at) => self.tasks[ti].tabs[at].name = name,
            }
            self.dirty = true;
        }
    }

    pub fn cancel_rename(&mut self) { self.rename = None; }

    /// Query daemon for idle status of all tabs (call every ~3s)
    pub fn refresh_idle_status(&mut self) {
        const IDLE_THRESHOLD_SECS: u64 = 30;
        let socket_path = crate::utils::get_socket_path();
        use std::os::unix::net::UnixStream;
        use std::io::{Write, BufRead, BufReader};
        let Ok(mut stream) = UnixStream::connect(&socket_path) else { return };
        let msg = serde_json::to_vec(&ClientMsg::GetIdleStatus).unwrap();
        let _ = stream.write_all(&msg);
        let _ = stream.write_all(b"\n");
        let _ = stream.flush();
        let mut line = String::new();
        let _ = BufReader::new(&stream).read_line(&mut line);
        if let Ok(ServerMsg::IdleStatus { tabs }) = serde_json::from_str(&line) {
            self.idle_tabs = tabs.into_iter()
                .filter(|(_, secs)| *secs >= IDLE_THRESHOLD_SECS)
                .map(|(id, _)| id)
                .collect();
        }
    }
}
