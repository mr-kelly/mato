use crate::utils::new_id;

use super::app::TabEntry;

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
        self.active_tab = self.active_tab.min(self.tabs.len().saturating_sub(1));
    }

    pub fn resize_all_ptys(&mut self, rows: u16, cols: u16) {
        for tab in &mut self.tabs {
            tab.resize_pty(rows, cols);
        }
    }
}
