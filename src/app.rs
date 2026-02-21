use std::{io::{Read, Write}, sync::{Arc, Mutex}, thread};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use ratatui::{layout::Rect, widgets::ListState};
use crate::{id::new_id, persistence::load_state};

#[derive(PartialEq, Clone, Copy)]
pub enum Focus { Sidebar, Topbar, Content }

#[derive(PartialEq, Clone, Copy)]
pub enum EscMode { None, Pending }

/// Rename mode: which area is being renamed + current buffer
#[derive(PartialEq, Clone)]
pub enum RenameTarget { Task(usize), Tab(usize, usize) } // (task_idx, tab_idx)

pub struct PtyState {
    pub writer: Box<dyn Write + Send>,
    pub parser: Arc<Mutex<vt100::Parser>>,
    pub master: Box<dyn portable_pty::MasterPty + Send>,
    pub _child: Box<dyn portable_pty::Child + Send + Sync>,
}

pub struct TabEntry {
    pub id: String,
    pub name: String,
    pub pty: Option<PtyState>,
}

impl TabEntry {
    pub fn new(name: impl Into<String>) -> Self {
        Self { id: new_id(), name: name.into(), pty: None }
    }
    pub fn with_id(id: String, name: impl Into<String>) -> Self {
        Self { id, name: name.into(), pty: None }
    }

    pub fn spawn_pty(&mut self, rows: u16, cols: u16) {
        if self.pty.is_some() { return; }
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 }).expect("openpty");
        let mut cmd = CommandBuilder::new("bash");
        cmd.env("TERM", "xterm-256color");
        let child = pair.slave.spawn_command(cmd).expect("spawn");
        let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 0)));
        let p2 = Arc::clone(&parser);
        let mut reader = pair.master.try_clone_reader().expect("reader");
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop { match reader.read(&mut buf) { Ok(0) | Err(_) => break, Ok(n) => p2.lock().unwrap().process(&buf[..n]) } }
        });
        self.pty = Some(PtyState { writer: pair.master.take_writer().expect("writer"), parser, master: pair.master, _child: child });
    }

    pub fn resize_pty(&mut self, rows: u16, cols: u16) {
        if let Some(p) = &mut self.pty {
            let _ = p.master.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 });
            p.parser.lock().unwrap().set_size(rows, cols);
        }
    }

    pub fn pty_write(&mut self, bytes: &[u8]) {
        if let Some(p) = &mut self.pty { let _ = p.writer.write_all(bytes); let _ = p.writer.flush(); }
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
    pub rename: Option<(RenameTarget, String)>, // active rename + buffer
    pub term_rows: u16,
    pub term_cols: u16,
    pub dirty: bool,
    // layout rects
    pub sidebar_list_area: Rect,
    pub sidebar_area: Rect,
    pub topbar_area: Rect,
    pub content_area: Rect,
    pub new_task_area: Rect,
    pub tab_areas: Vec<Rect>,   // one per tab in current task
    pub new_tab_area: Rect,
    pub last_click: Option<(u16, u16, std::time::Instant)>,
}

impl App {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        let tasks = if let Some(s) = load_state() {
            s.tasks.into_iter().map(|t| {
                let tabs = t.tabs.into_iter().map(|tb| TabEntry::with_id(tb.id, tb.name)).collect();
                Task { id: t.id, name: t.name, tabs, active_tab: t.active_tab }
            }).collect()
        } else {
            vec![Task::new("Task 1")]
        };
        Self {
            tasks, list_state,
            focus: Focus::Sidebar, esc_mode: EscMode::None, rename: None,
            term_rows: 24, term_cols: 80, dirty: false,
            sidebar_list_area: Rect::default(), sidebar_area: Rect::default(),
            topbar_area: Rect::default(), content_area: Rect::default(),
            new_task_area: Rect::default(), tab_areas: vec![], new_tab_area: Rect::default(),
            last_click: None,
        }
    }

    pub fn selected(&self) -> usize { self.list_state.selected().unwrap_or(0) }

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
}
