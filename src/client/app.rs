use crate::protocol::{ClientMsg, ServerMsg};
use crate::theme::ThemeColors;
use crate::{
    client::persistence::{load_state, SavedState},
    providers::DaemonProvider,
    terminal_provider::TerminalProvider,
    utils::new_id,
};
use ratatui::{layout::Rect, widgets::ListState};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Focus {
    Sidebar,
    Topbar,
    Content,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum JumpMode {
    None,
    Active, // ESC pressed in Content - can jump OR use arrows
}

pub const JUMP_LABELS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

#[derive(PartialEq, Clone)]
pub enum RenameTarget {
    Desk(usize),
    Tab(usize, usize),
    Office(usize),
}

pub struct OfficeSelectorState {
    pub active: bool,
    pub list_state: ListState,
}

impl OfficeSelectorState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            active: false,
            list_state,
        }
    }
}

impl Default for OfficeSelectorState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OfficeDeleteConfirm {
    pub office_idx: usize,
    pub input: String,
}

impl OfficeDeleteConfirm {
    pub fn new(office_idx: usize) -> Self {
        Self {
            office_idx,
            input: String::new(),
        }
    }
}

struct MouseModeCache {
    tab_id: String,
    mouse_enabled: bool,
    checked_at: Instant,
}

pub struct TabEntry {
    pub id: String,
    pub name: String,
    pub provider: Box<dyn TerminalProvider>,
}

impl TabEntry {
    pub fn new(name: impl Into<String>) -> Self {
        let id = new_id();
        let socket_path = crate::utils::get_socket_path()
            .to_string_lossy()
            .to_string();
        Self {
            id: id.clone(),
            name: name.into(),
            provider: Box::new(DaemonProvider::new(id, socket_path)),
        }
    }
    pub fn with_id(id: String, name: impl Into<String>) -> Self {
        let socket_path = crate::utils::get_socket_path()
            .to_string_lossy()
            .to_string();
        Self {
            id: id.clone(),
            name: name.into(),
            provider: Box::new(DaemonProvider::new(id, socket_path)),
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

pub struct Desk {
    pub id: String,
    pub name: String,
    pub tabs: Vec<TabEntry>,
    pub active_tab: usize,
}

impl Desk {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: new_id(),
            name: name.into(),
            tabs: vec![TabEntry::new("Terminal 1")],
            active_tab: 0,
        }
    }

    pub fn active_tab_ref(&self) -> &TabEntry {
        &self.tabs[self.active_tab]
    }

    pub fn new_tab(&mut self) {
        let n = self.tabs.len() + 1;
        self.tabs.push(TabEntry::new(format!("Terminal {n}")));
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }

        // Send ClosePty message to daemon before removing tab
        let tab = &self.tabs[self.active_tab];
        let socket_path = crate::utils::get_socket_path();
        if let Ok(mut stream) = std::os::unix::net::UnixStream::connect(&socket_path) {
            use crate::protocol::ClientMsg;
            use std::io::Write;
            let msg = ClientMsg::ClosePty {
                tab_id: tab.id.clone(),
            };
            if let Ok(json) = serde_json::to_vec(&msg) {
                let _ = stream.write_all(&json);
                let _ = stream.write_all(b"\n");
                let _ = stream.flush();
            }
        }

        self.tabs.remove(self.active_tab);
        self.active_tab = self.active_tab.min(self.tabs.len() - 1);
    }

    pub fn resize_all_ptys(&mut self, rows: u16, cols: u16) {
        for tab in &mut self.tabs {
            tab.resize_pty(rows, cols);
        }
    }
}

pub struct Office {
    pub id: String,
    pub name: String,
    pub desks: Vec<Desk>,
    pub active_desk: usize,
}

impl Office {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: new_id(),
            name: name.into(),
            desks: vec![Desk::new("Desk 1")],
            active_desk: 0,
        }
    }
}

pub struct App {
    pub offices: Vec<Office>,
    pub current_office: usize,
    pub list_state: ListState,
    pub focus: Focus,
    pub prev_focus: Focus,
    pub jump_mode: JumpMode,
    pub rename: Option<(RenameTarget, String)>,
    pub office_selector: OfficeSelectorState,
    pub term_rows: u16,
    pub term_cols: u16,
    pub pending_resize: Option<(u16, u16, std::time::Instant)>, // Delay resize to avoid content loss
    pub dirty: bool,
    // layout rects
    pub sidebar_list_area: Rect,
    pub sidebar_area: Rect,
    pub topbar_area: Rect,
    pub content_area: Rect,
    pub new_desk_area: Rect,
    pub tab_areas: Vec<Rect>,
    pub tab_area_tab_indices: Vec<usize>,
    pub new_tab_area: Rect,
    pub tab_scroll: usize,
    pub last_click: Option<(u16, u16, std::time::Instant)>,
    /// tab_ids that are ACTIVE (have output in last 2 seconds)
    pub active_tabs: HashSet<String>,
    pub daemon_connected: bool,
    pub daemon_last_ok: Instant,
    pub terminal_titles: HashMap<String, String>,
    /// Spinner animation state
    pub spinner_frame: usize,
    pub last_spinner_update: Instant,
    pub last_title_sync: Instant,
    pub theme: ThemeColors,
    /// Settings screen open
    pub show_settings: bool,
    pub settings_selected: usize,
    /// Some(version) if an update is available
    pub update_available: Option<String>,
    pub last_update_check: Instant,
    /// Trigger onboarding for new office
    pub should_show_onboarding: bool,
    /// Full-screen content-only copy/scroll mode.
    pub copy_mode: bool,
    /// Office delete confirmation
    pub office_delete_confirm: Option<OfficeDeleteConfirm>,
    mouse_mode_cache: Option<MouseModeCache>,
    active_status_rx: Option<Receiver<HashSet<String>>>,
    tab_switch_started_at: Option<Instant>,
    pub pending_bell: bool,
    /// Timestamp of last ESC press in Content focus (for double-ESC detection)
    pub last_content_esc: Option<Instant>,
    /// Frame generation for screen cache change detection
    pub last_rendered_screen_gen: u64,
}

impl App {
    fn jump_key_reserved_for_focus(&self, c: char) -> bool {
        let key = c.to_ascii_lowercase();
        match self.focus {
            Focus::Content => matches!(key, 'c' | 'r' | 'q'),
            Focus::Sidebar | Focus::Topbar => matches!(key, 'r' | 'q'),
        }
    }

    pub fn jump_labels(&self) -> Vec<char> {
        JUMP_LABELS
            .chars()
            .filter(|c| !self.jump_key_reserved_for_focus(*c))
            .collect()
    }

    fn visible_desk_indices(&self) -> Vec<usize> {
        let desks_len = self.offices[self.current_office].desks.len();
        if desks_len == 0 {
            return vec![];
        }
        let visible_rows = self.sidebar_list_area.height.saturating_sub(2) as usize;
        if visible_rows == 0 {
            return (0..desks_len).collect();
        }
        let start = self.list_state.offset().min(desks_len.saturating_sub(1));
        let end = (start + visible_rows).min(desks_len);
        (start..end).collect()
    }

    pub fn new() -> Self {
        let mut list_state = ListState::default();
        let (offices, current_office): (Vec<Office>, usize) = if let Ok(s) = load_state() {
            let offices: Vec<Office> = s
                .offices
                .into_iter()
                .map(|o| {
                    let desks = o
                        .desks
                        .into_iter()
                        .map(|d| {
                            let tabs = d
                                .tabs
                                .into_iter()
                                .map(|tb| TabEntry::with_id(tb.id, tb.name))
                                .collect();
                            Desk {
                                id: d.id,
                                name: d.name,
                                tabs,
                                active_tab: d.active_tab,
                            }
                        })
                        .collect();
                    Office {
                        id: o.id,
                        name: o.name,
                        desks,
                        active_desk: o.active_desk,
                    }
                })
                .collect();
            if offices.is_empty() {
                (vec![Office::new("Default")], 0)
            } else {
                (offices, s.current_office)
            }
        } else {
            (vec![Office::new("Default")], 0)
        };
        let current_office = current_office.min(offices.len().saturating_sub(1));
        let active_desk = offices[current_office]
            .active_desk
            .min(offices[current_office].desks.len().saturating_sub(1));
        let mut offices = offices;
        offices[current_office].active_desk = active_desk;
        list_state.select(Some(active_desk));
        Self {
            offices,
            current_office,
            list_state,
            focus: Focus::Sidebar,
            prev_focus: Focus::Sidebar,
            jump_mode: JumpMode::None,
            rename: None,
            office_selector: OfficeSelectorState::new(),
            term_rows: 24,
            term_cols: 80,
            pending_resize: None,
            dirty: false,
            sidebar_list_area: Rect::default(),
            sidebar_area: Rect::default(),
            topbar_area: Rect::default(),
            content_area: Rect::default(),
            new_desk_area: Rect::default(),
            tab_areas: vec![],
            new_tab_area: Rect::default(),
            tab_area_tab_indices: vec![],
            tab_scroll: 0,
            last_click: None,
            active_tabs: HashSet::new(),
            daemon_connected: false,
            daemon_last_ok: Instant::now(),
            terminal_titles: HashMap::new(),
            spinner_frame: 0,
            last_spinner_update: Instant::now(),
            last_title_sync: Instant::now() - Duration::from_millis(500),
            theme: crate::theme::load(),
            show_settings: false,
            settings_selected: crate::theme::selected_index(),
            update_available: None,
            // Force first update check immediately after startup.
            last_update_check: Instant::now() - std::time::Duration::from_secs(3601),
            should_show_onboarding: false,
            copy_mode: false,
            office_delete_confirm: None,
            mouse_mode_cache: None,
            active_status_rx: None,
            tab_switch_started_at: None,
            pending_bell: false,
            last_content_esc: None,
            last_rendered_screen_gen: 0,
        }
    }

    #[allow(dead_code)]
    pub fn from_saved(state: SavedState) -> Self {
        let mut list_state = ListState::default();
        let mut offices = state
            .offices
            .into_iter()
            .map(|o| {
                let desks = o
                    .desks
                    .into_iter()
                    .map(|d| {
                        let tabs = d
                            .tabs
                            .into_iter()
                            .map(|tb| TabEntry::with_id(tb.id, tb.name))
                            .collect();
                        Desk {
                            id: d.id,
                            name: d.name,
                            tabs,
                            active_tab: d.active_tab,
                        }
                    })
                    .collect();
                Office {
                    id: o.id,
                    name: o.name,
                    desks,
                    active_desk: o.active_desk,
                }
            })
            .collect::<Vec<_>>();
        if offices.is_empty() {
            offices.push(Office::new("Default"));
        }
        let current_office = state.current_office.min(offices.len().saturating_sub(1));
        let active_desk = offices[current_office]
            .active_desk
            .min(offices[current_office].desks.len().saturating_sub(1));
        offices[current_office].active_desk = active_desk;
        list_state.select(Some(active_desk));

        Self {
            offices,
            current_office,
            list_state,
            focus: Focus::Sidebar,
            prev_focus: Focus::Sidebar,
            jump_mode: JumpMode::None,
            rename: None,
            office_selector: OfficeSelectorState::new(),
            term_rows: 24,
            term_cols: 80,
            pending_resize: None,
            dirty: false,
            sidebar_list_area: Rect::default(),
            sidebar_area: Rect::default(),
            topbar_area: Rect::default(),
            content_area: Rect::default(),
            new_desk_area: Rect::default(),
            tab_areas: vec![],
            new_tab_area: Rect::default(),
            tab_area_tab_indices: vec![],
            tab_scroll: 0,
            last_click: None,
            active_tabs: HashSet::new(),
            daemon_connected: false,
            daemon_last_ok: Instant::now(),
            terminal_titles: HashMap::new(),
            spinner_frame: 0,
            last_spinner_update: Instant::now(),
            last_title_sync: Instant::now() - Duration::from_millis(500),
            theme: crate::theme::load(),
            show_settings: false,
            settings_selected: crate::theme::selected_index(),
            update_available: None,
            // Force first update check immediately after startup.
            last_update_check: Instant::now() - std::time::Duration::from_secs(3601),
            should_show_onboarding: false,
            copy_mode: false,
            office_delete_confirm: None,
            mouse_mode_cache: None,
            active_status_rx: None,
            tab_switch_started_at: None,
            pending_bell: false,
            last_content_esc: None,
            last_rendered_screen_gen: 0,
        }
    }

    // Helper methods to access current office's desks
    #[allow(dead_code)]
    pub fn desks(&self) -> &Vec<Desk> {
        &self.offices[self.current_office].desks
    }
    #[allow(dead_code)]
    pub fn desks_mut(&mut self) -> &mut Vec<Desk> {
        &mut self.offices[self.current_office].desks
    }
    #[allow(dead_code)]
    pub fn office(&self) -> &Office {
        &self.offices[self.current_office]
    }
    #[allow(dead_code)]
    pub fn office_mut(&mut self) -> &mut Office {
        &mut self.offices[self.current_office]
    }

    pub fn selected(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    pub fn select_desk(&mut self, desk_idx: usize) {
        if self.offices.is_empty() {
            self.list_state.select(None);
            return;
        }
        let max = self.offices[self.current_office]
            .desks
            .len()
            .saturating_sub(1);
        let idx = desk_idx.min(max);
        let prev = self.selected();
        self.list_state.select(Some(idx));
        self.offices[self.current_office].active_desk = idx;
        if prev != idx {
            self.dirty = true;
        }
    }

    #[allow(dead_code)]
    pub fn active_desk(&self) -> usize {
        self.selected()
    }

    pub fn cur_desk_mut(&mut self) -> &mut Desk {
        let i = self.selected();
        &mut self.offices[self.current_office].desks[i]
    }

    pub fn switch_office(&mut self, office_idx: usize) {
        if office_idx < self.offices.len() {
            self.current_office = office_idx;
            let active_desk = self.offices[office_idx].active_desk;
            self.select_desk(active_desk);
            self.tab_scroll = 0;
            self.mark_tab_switch();
            self.spawn_active_pty();
            self.dirty = true;
        }
    }

    pub fn new_desk(&mut self) {
        let n = self.offices[self.current_office].desks.len() + 1;
        self.offices[self.current_office]
            .desks
            .push(Desk::new(format!("Desk {n}")));
        self.select_desk(self.offices[self.current_office].desks.len() - 1);
        self.dirty = true;
    }

    pub fn close_desk(&mut self) {
        if self.offices[self.current_office].desks.len() <= 1 {
            return;
        }
        let idx = self.selected();

        // Close all PTYs in this desk
        let desk = &self.offices[self.current_office].desks[idx];
        let socket_path = crate::utils::get_socket_path();
        if let Ok(mut stream) = std::os::unix::net::UnixStream::connect(&socket_path) {
            use crate::protocol::ClientMsg;
            use std::io::Write;
            for tab in &desk.tabs {
                let msg = ClientMsg::ClosePty {
                    tab_id: tab.id.clone(),
                };
                if let Ok(json) = serde_json::to_vec(&msg) {
                    let _ = stream.write_all(&json);
                    let _ = stream.write_all(b"\n");
                }
            }
            let _ = stream.flush();
        }

        self.offices[self.current_office].desks.remove(idx);
        self.select_desk(idx.min(self.offices[self.current_office].desks.len() - 1));
        self.dirty = true;
    }

    pub fn nav(&mut self, delta: i32) {
        let max = self.offices[self.current_office]
            .desks
            .len()
            .saturating_sub(1) as i32;
        let next = (self.selected() as i32 + delta).clamp(0, max) as usize;
        let changed = self.selected() != next;
        self.select_desk(next);
        self.tab_scroll = 0;
        if changed {
            self.mark_tab_switch();
            self.spawn_active_pty();
            self.dirty = true;
        }
    }

    pub fn spawn_active_pty(&mut self) {
        let (rows, cols) = (self.term_rows, self.term_cols);
        let i = self.selected();
        let at = self.offices[self.current_office].desks[i].active_tab;
        self.offices[self.current_office].desks[i].tabs[at].spawn_pty(rows, cols);
    }

    pub fn restart_active_pty(&mut self) {
        let i = self.selected();
        let at = self.offices[self.current_office].desks[i].active_tab;
        let tab_id = self.offices[self.current_office].desks[i].tabs[at]
            .id
            .clone();

        let socket_path = crate::utils::get_socket_path();
        if let Ok(mut stream) = std::os::unix::net::UnixStream::connect(&socket_path) {
            use crate::protocol::ClientMsg;
            use std::io::Write;
            let msg = ClientMsg::ClosePty {
                tab_id: tab_id.clone(),
            };
            if let Ok(json) = serde_json::to_vec(&msg) {
                let _ = stream.write_all(&json);
                let _ = stream.write_all(b"\n");
                let _ = stream.flush();
            }
        }

        self.spawn_active_pty();
        self.mark_tab_switch();
    }

    pub fn resize_all_ptys(&mut self, rows: u16, cols: u16) {
        // Don't resize immediately - wait for user to stop resizing
        // This prevents content loss during window resize
        if self.term_rows != rows || self.term_cols != cols {
            self.pending_resize = Some((rows, cols, std::time::Instant::now()));
        }
    }

    pub fn apply_pending_resize(&mut self) {
        if let Some((rows, cols, time)) = self.pending_resize {
            // Wait 500ms after last resize before applying
            if time.elapsed().as_millis() > 500 {
                if self.term_rows != rows || self.term_cols != cols {
                    tracing::info!(
                        "Applying delayed resize: {}x{} -> {}x{}",
                        self.term_rows,
                        self.term_cols,
                        rows,
                        cols
                    );
                    self.term_rows = rows;
                    self.term_cols = cols;
                    for desk in &mut self.offices[self.current_office].desks {
                        desk.resize_all_ptys(rows, cols);
                    }
                }
                self.pending_resize = None;
            }
        }
    }

    pub fn pty_write(&mut self, bytes: &[u8]) {
        let i = self.selected();
        let at = self.offices[self.current_office].desks[i].active_tab;
        self.offices[self.current_office].desks[i].tabs[at].pty_write(bytes);
    }

    pub fn pty_paste(&mut self, text: &str) {
        let i = self.selected();
        let at = self.offices[self.current_office].desks[i].active_tab;
        self.offices[self.current_office].desks[i].tabs[at]
            .provider
            .paste(text);
    }

    pub fn pty_scroll(&mut self, delta: i32) {
        let i = self.selected();
        let at = self.offices[self.current_office].desks[i].active_tab;
        self.offices[self.current_office].desks[i].tabs[at]
            .provider
            .scroll(delta);
    }

    pub fn active_provider_screen_generation(&self) -> u64 {
        let i = self.selected();
        let desk = &self.offices[self.current_office].desks[i];
        if desk.tabs.is_empty() {
            return 0;
        }
        desk.tabs[desk.active_tab].provider.screen_generation()
    }

    pub fn pty_mouse_mode_enabled(&mut self) -> bool {
        let i = self.selected();
        let at = self.offices[self.current_office].desks[i].active_tab;
        let tab_id = self.offices[self.current_office].desks[i].tabs[at]
            .id
            .clone();
        if let Some(cache) = &self.mouse_mode_cache {
            if cache.tab_id == tab_id && cache.checked_at.elapsed() < Duration::from_millis(100) {
                return cache.mouse_enabled;
            }
        }

        let enabled = self.offices[self.current_office].desks[i].tabs[at]
            .provider
            .mouse_mode_enabled();
        self.mouse_mode_cache = Some(MouseModeCache {
            tab_id,
            mouse_enabled: enabled,
            checked_at: Instant::now(),
        });
        enabled
    }

    /// Send focus in/out events to PTY when focus changes
    pub fn sync_focus_events(&mut self) {
        if self.prev_focus == self.focus {
            return;
        }
        let was_content = self.prev_focus == Focus::Content;
        let is_content = self.focus == Focus::Content;
        if is_content {
            self.pty_write(b"\x1b[I");
        } // focus in
        if was_content {
            self.pty_write(b"\x1b[O");
        } // focus out
        self.prev_focus = self.focus;
    }
    pub fn sync_tab_titles(&mut self) {
        if self.last_title_sync.elapsed() < Duration::from_millis(500) {
            return;
        }
        self.last_title_sync = Instant::now();

        // Sync only the currently visible tab to avoid per-frame N×socket round-trips.
        let desk_idx = self.selected();
        let tab_idx = self.offices[self.current_office].desks[desk_idx].active_tab;
        if let Some(tab) = self.offices[self.current_office].desks[desk_idx]
            .tabs
            .get_mut(tab_idx)
        {
            let rows = self.term_rows.max(1);
            let cols = self.term_cols.max(1);
            let screen = tab.provider.get_screen(rows, cols);
            if let Some(title) = screen.title {
                if !title.is_empty() {
                    self.terminal_titles.insert(tab.id.clone(), title);
                }
            }
        }
    }

    /// Start renaming task at sidebar index
    pub fn begin_rename_desk(&mut self, idx: usize) {
        let name = self.offices[self.current_office].desks[idx].name.clone();
        self.rename = Some((RenameTarget::Desk(idx), name));
    }

    /// Start renaming active tab of current task
    pub fn begin_rename_tab(&mut self) {
        let ti = self.selected();
        let at = self.offices[self.current_office].desks[ti].active_tab;
        let name = self.offices[self.current_office].desks[ti].tabs[at]
            .name
            .clone();
        self.rename = Some((RenameTarget::Tab(ti, at), name));
    }

    pub fn commit_rename(&mut self) {
        if let Some((target, buf)) = self.rename.take() {
            let name = buf.trim().to_string();
            if name.is_empty() {
                return;
            }
            match target {
                RenameTarget::Desk(i) => self.offices[self.current_office].desks[i].name = name,
                RenameTarget::Tab(ti, at) => {
                    self.offices[self.current_office].desks[ti].tabs[at].name = name
                }
                RenameTarget::Office(i) => self.offices[i].name = name,
            }
            self.dirty = true;
        }
    }

    pub fn cancel_rename(&mut self) {
        self.rename = None;
    }

    /// Handle jump mode character selection
    pub fn handle_jump_selection(&mut self, c: char) {
        let targets = self.jump_targets();
        let labels = self.jump_labels();
        let origin_focus = self.focus;

        // Map character to target
        if let Some(idx) = labels.iter().position(|&ch| ch == c) {
            if idx < targets.len() {
                let (kind, task_idx, tab_idx) = targets[idx];
                match kind {
                    't' => {
                        // Jump to desk target.
                        self.select_desk(task_idx);
                        self.focus = match origin_focus {
                            Focus::Content => Focus::Content,
                            Focus::Topbar => Focus::Sidebar,
                            Focus::Sidebar => Focus::Sidebar,
                        };
                        self.mark_tab_switch();
                        self.spawn_active_pty();
                    }
                    'b' => {
                        // Jump to tab target.
                        self.offices[self.current_office].desks[task_idx].active_tab = tab_idx;
                        self.focus = match origin_focus {
                            Focus::Content => Focus::Content,
                            Focus::Sidebar => Focus::Topbar,
                            Focus::Topbar => Focus::Topbar,
                        };
                        self.mark_tab_switch();
                        self.spawn_active_pty();
                    }
                    _ => {}
                }
                self.dirty = true;
            }
        }

        // Exit jump mode
        self.jump_mode = JumpMode::None;
    }

    pub fn jump_targets(&self) -> Vec<(char, usize, usize)> {
        let max_labels = self.jump_labels().len();
        let mut targets: Vec<(char, usize, usize)> = Vec::new();
        let desk_indices = self.visible_desk_indices();
        let tab_indices: Vec<usize> = self.tab_area_tab_indices.clone();
        let task_idx = self.selected();

        let push_tab = |targets: &mut Vec<(char, usize, usize)>, t: usize| {
            targets.push(('b', task_idx, t));
        };
        let push_desk = |targets: &mut Vec<(char, usize, usize)>, d: usize| {
            targets.push(('t', d, 0));
        };

        match self.focus {
            Focus::Topbar => {
                for &t in &tab_indices {
                    if targets.len() >= max_labels {
                        break;
                    }
                    push_tab(&mut targets, t);
                }
                for &d in &desk_indices {
                    if targets.len() >= max_labels {
                        break;
                    }
                    push_desk(&mut targets, d);
                }
            }
            Focus::Sidebar => {
                for &d in &desk_indices {
                    if targets.len() >= max_labels {
                        break;
                    }
                    push_desk(&mut targets, d);
                }
                for &t in &tab_indices {
                    if targets.len() >= max_labels {
                        break;
                    }
                    push_tab(&mut targets, t);
                }
            }
            Focus::Content => {
                // Balanced allocation: interleave tab/desk as much as possible.
                let (mut ti, mut di) = (0usize, 0usize);
                let mut take_tab = true;
                while targets.len() < max_labels
                    && (ti < tab_indices.len() || di < desk_indices.len())
                {
                    if take_tab {
                        if ti < tab_indices.len() {
                            push_tab(&mut targets, tab_indices[ti]);
                            ti += 1;
                        } else if di < desk_indices.len() {
                            push_desk(&mut targets, desk_indices[di]);
                            di += 1;
                        }
                    } else if di < desk_indices.len() {
                        push_desk(&mut targets, desk_indices[di]);
                        di += 1;
                    } else if ti < tab_indices.len() {
                        push_tab(&mut targets, tab_indices[ti]);
                        ti += 1;
                    }
                    take_tab = !take_tab;
                }
            }
        }

        targets
    }

    /// Query daemon for active status of all tabs (call every frame)
    pub fn refresh_active_status(&mut self) {
        const ACTIVE_THRESHOLD_SECS: u64 = 2;
        if self.active_status_rx.is_none() {
            self.active_status_rx = Some(Self::spawn_active_status_worker(ACTIVE_THRESHOLD_SECS));
            self.daemon_connected = false;
        }
        let mut latest: Option<HashSet<String>> = None;
        if let Some(rx) = &self.active_status_rx {
            while let Ok(active) = rx.try_recv() {
                latest = Some(active);
            }
        }
        if let Some(active) = latest {
            self.active_tabs = active;
            self.daemon_connected = true;
            self.daemon_last_ok = Instant::now();
        } else if self.daemon_connected && self.daemon_last_ok.elapsed() > Duration::from_secs(2) {
            // No recent successful daemon status reads.
            self.daemon_connected = false;
        }
    }

    fn spawn_active_status_worker(active_threshold_secs: u64) -> Receiver<HashSet<String>> {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let socket_path = crate::utils::get_socket_path();
            loop {
                let active = (|| -> Option<HashSet<String>> {
                    let mut stream = UnixStream::connect(&socket_path).ok()?;
                    stream
                        .set_read_timeout(Some(Duration::from_millis(150)))
                        .ok()?;
                    let msg = serde_json::to_vec(&ClientMsg::GetIdleStatus).ok()?;
                    stream.write_all(&msg).ok()?;
                    stream.write_all(b"\n").ok()?;
                    stream.flush().ok()?;
                    let mut line = String::new();
                    BufReader::new(&stream).read_line(&mut line).ok()?;
                    let ServerMsg::IdleStatus { tabs } = serde_json::from_str(&line).ok()? else {
                        return None;
                    };
                    Some(
                        tabs.into_iter()
                            .filter(|(_, secs)| *secs < active_threshold_secs)
                            .map(|(id, _)| id)
                            .collect(),
                    )
                })();
                let poll_interval = if let Some(active_tabs) = active {
                    let next = if active_tabs.is_empty() {
                        Duration::from_millis(1000)
                    } else {
                        Duration::from_millis(300)
                    };
                    if tx.send(active_tabs).is_err() {
                        break;
                    }
                    next
                } else {
                    // Daemon/socket issue: back off briefly to avoid hot-looping.
                    Duration::from_millis(1000)
                };
                thread::sleep(poll_interval);
            }
        });
        rx
    }

    pub fn mark_tab_switch(&mut self) {
        self.tab_switch_started_at = Some(Instant::now());
    }

    pub fn finish_tab_switch_measurement(&mut self) -> Option<Duration> {
        self.tab_switch_started_at.take().map(|t| t.elapsed())
    }

    pub fn refresh_update_status(&mut self) {
        let socket_path = crate::utils::get_socket_path();
        self.refresh_update_status_from_socket(socket_path);
    }

    pub fn refresh_update_status_from_socket<P: AsRef<Path>>(&mut self, socket_path: P) {
        use std::time::Duration;
        if self.last_update_check.elapsed() < Duration::from_secs(3600) {
            return;
        }
        self.last_update_check = Instant::now();
        use std::io::{BufRead, BufReader, Write};
        use std::os::unix::net::UnixStream;
        let Ok(mut stream) = UnixStream::connect(socket_path.as_ref()) else {
            return;
        };
        let msg = serde_json::to_vec(&ClientMsg::GetUpdateStatus).unwrap();
        let _ = stream.write_all(&msg);
        let _ = stream.write_all(b"\n");
        let _ = stream.flush();
        let mut line = String::new();
        let _ = BufReader::new(&stream).read_line(&mut line);
        if let Ok(ServerMsg::UpdateStatus { latest }) = serde_json::from_str(&line) {
            self.update_available = latest;
        }
    }

    /// Update spinner animation frame
    pub fn update_spinner(&mut self) {
        use std::time::Duration;
        if self.last_spinner_update.elapsed() > Duration::from_millis(80) {
            self.spinner_frame = (self.spinner_frame + 1) % 10;
            self.last_spinner_update = Instant::now();
        }
    }

    /// Get current spinner character
    pub fn get_spinner(&self) -> &str {
        const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        SPINNER[self.spinner_frame]
    }

    /// Check if any tab is active
    pub fn has_active_tabs(&self) -> bool {
        !self.active_tabs.is_empty()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
