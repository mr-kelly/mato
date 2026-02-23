use super::app::App;
use crate::protocol::{ClientMsg, ServerMsg};
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

impl App {
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

    pub fn refresh_update_status(&mut self) {
        let socket_path = crate::utils::get_socket_path();
        self.refresh_update_status_from_socket(socket_path);
    }

    pub fn refresh_update_status_from_socket<P: AsRef<Path>>(&mut self, socket_path: P) {
        if self.last_update_check.elapsed() < Duration::from_secs(3600) {
            return;
        }
        self.last_update_check = Instant::now();
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
}
