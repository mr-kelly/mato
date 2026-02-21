use crate::terminal_provider::{TerminalProvider, ScreenContent};
use crate::protocol::{ClientMsg, ServerMsg};
use std::sync::{Arc, Mutex};
use std::os::unix::net::UnixStream as StdUnixStream;
use std::io::{Write, BufReader, BufRead};

pub struct DaemonProvider {
    tab_id: String,
    socket_path: String,
    stream: Arc<Mutex<Option<StdUnixStream>>>,
}

impl DaemonProvider {
    pub fn new(tab_id: String, socket_path: String) -> Self {
        Self {
            tab_id,
            socket_path,
            stream: Arc::new(Mutex::new(None)),
        }
    }

    fn send_msg(&self, msg: ClientMsg) -> Option<ServerMsg> {
        let mut guard = self.stream.lock().unwrap();
        
        if guard.is_none() {
            match StdUnixStream::connect(&self.socket_path) {
                Ok(s) => *guard = Some(s),
                Err(_) => return None,
            }
        }
        
        let stream = guard.as_mut()?;
        let json = serde_json::to_vec(&msg).ok()?;
        stream.write_all(&json).ok()?;
        stream.write_all(b"\n").ok()?;
        stream.flush().ok()?;

        // Read line-by-line
        let mut reader = BufReader::new(stream.try_clone().ok()?);
        let mut line = String::new();
        reader.read_line(&mut line).ok()?;
        serde_json::from_str(&line).ok()
    }
    
    // Send message without waiting for response (fire and forget)
    fn send_msg_no_response(&self, msg: ClientMsg) {
        let mut guard = self.stream.lock().unwrap();
        
        if guard.is_none() {
            match StdUnixStream::connect(&self.socket_path) {
                Ok(s) => *guard = Some(s),
                Err(_) => return,
            }
        }
        
        if let Some(stream) = guard.as_mut() {
            let json = serde_json::to_vec(&msg).unwrap();
            let _ = stream.write_all(&json);
            let _ = stream.write_all(b"\n");
            let _ = stream.flush();
        }
    }
}

impl TerminalProvider for DaemonProvider {
    fn spawn(&mut self, rows: u16, cols: u16) {
        self.send_msg(ClientMsg::Spawn {
            tab_id: self.tab_id.clone(),
            rows,
            cols,
        });
    }

    fn resize(&mut self, rows: u16, cols: u16) {
        // Fire and forget - no response needed
        self.send_msg_no_response(ClientMsg::Resize {
            tab_id: self.tab_id.clone(),
            rows,
            cols,
        });
    }

    fn write(&mut self, bytes: &[u8]) {
        // Fire and forget - no response needed
        self.send_msg_no_response(ClientMsg::Input {
            tab_id: self.tab_id.clone(),
            data: bytes.to_vec(),
        });
    }

    fn get_screen(&self, rows: u16, cols: u16) -> ScreenContent {
        match self.send_msg(ClientMsg::GetScreen {
            tab_id: self.tab_id.clone(),
            rows,
            cols,
        }) {
            Some(ServerMsg::Screen { content, .. }) => content,
            Some(ServerMsg::Error { message }) => {
                tracing::error!("GetScreen error for tab {}: {}", self.tab_id, message);
                ScreenContent::default()
            }
            _ => ScreenContent::default(),
        }
    }
}
