use crate::theme::ThemeColors;
use crate::{
    client::persistence::{load_state, SavedState},
    providers::DaemonProvider,
    terminal_provider::TerminalProvider,
    utils::new_id,
};
use ratatui::{layout::Rect, widgets::ListState};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Receiver;
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
pub const CONTENT_ESC_DOUBLE_PRESS_WINDOW_MS: u64 = 300;

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

pub struct DeskDeleteConfirm {
    pub desk_idx: usize,
}

impl DeskDeleteConfirm {
    pub fn new(desk_idx: usize) -> Self {
        Self { desk_idx }
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

pub use crate::client::desk::Desk;

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
    /// Desk delete confirmation (yes/no)
    pub desk_delete_confirm: Option<DeskDeleteConfirm>,
    mouse_mode_cache: Option<MouseModeCache>,
    pub(crate) active_status_rx: Option<Receiver<HashSet<String>>>,
    tab_switch_started_at: Option<Instant>,
    pub pending_bell: bool,
    /// Timestamp of last ESC press in Content focus (for double-ESC detection)
    pub last_content_esc: Option<Instant>,
    /// Frame generation for screen cache change detection
    pub last_rendered_screen_gen: u64,
}

impl App {
    pub fn flush_pending_content_esc(&mut self) {
        let Some(prev) = self.last_content_esc else {
            return;
        };
        if prev.elapsed() >= Duration::from_millis(CONTENT_ESC_DOUBLE_PRESS_WINDOW_MS) {
            self.last_content_esc = None;
            self.pty_write(b"\x1b");
        }
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
            desk_delete_confirm: None,
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
            desk_delete_confirm: None,
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
        self.spawn_active_pty();
        self.dirty = true;
    }

    pub fn request_close_desk(&mut self) {
        if self.offices[self.current_office].desks.len() <= 1 {
            return;
        }
        self.desk_delete_confirm = Some(DeskDeleteConfirm::new(self.selected()));
    }

    #[allow(dead_code)]
    pub fn close_desk(&mut self) {
        if self.offices[self.current_office].desks.len() <= 1 {
            return;
        }
        let idx = self.selected();
        self.close_desk_at(idx);
    }

    fn close_desk_at(&mut self, idx: usize) {
        if self.offices[self.current_office].desks.len() <= 1 {
            return;
        }
        if idx >= self.offices[self.current_office].desks.len() {
            return;
        }

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
        self.select_desk(idx.min(self.offices[self.current_office].desks.len().saturating_sub(1)));
        self.tab_scroll = 0;
        self.mark_tab_switch();
        self.spawn_active_pty();
        self.dirty = true;
    }

    pub fn confirm_close_desk(&mut self, desk_idx: usize) {
        self.desk_delete_confirm = None;
        self.close_desk_at(desk_idx);
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

    pub fn mark_tab_switch(&mut self) {
        self.tab_switch_started_at = Some(Instant::now());
    }

    pub fn finish_tab_switch_measurement(&mut self) -> Option<Duration> {
        self.tab_switch_started_at.take().map(|t| t.elapsed())
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
