use crate::protocol::{ClientMsg, ServerMsg};
use crate::terminal_provider::{ScreenContent, TerminalProvider};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream as StdUnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

struct ScreenCacheEntry {
    rows: u16,
    cols: u16,
    fetched_at: Instant,
    content: ScreenContent,
}

pub struct DaemonProvider {
    tab_id: String,
    socket_path: String,
    current_size: (u16, u16), // Track size to avoid unnecessary resizes
    screen_cache: Arc<Mutex<Option<ScreenCacheEntry>>>,
    screen_requested_size: Arc<Mutex<(u16, u16)>>,
    last_screen_request_at: Arc<Mutex<Instant>>,
    worker_running: Arc<AtomicBool>,
}

impl DaemonProvider {
    fn send_msg_on_stream(stream: &mut StdUnixStream, msg: &ClientMsg) -> Option<ServerMsg> {
        let json = serde_json::to_vec(msg).ok()?;
        stream.write_all(&json).ok()?;
        stream.write_all(b"\n").ok()?;
        stream.flush().ok()?;

        // Use a cloned fd for buffered line reads without taking ownership.
        let mut reader = BufReader::new(stream.try_clone().ok()?);
        let mut line = String::new();
        reader.read_line(&mut line).ok()?;
        serde_json::from_str(&line).ok()
    }

    pub fn new(tab_id: String, socket_path: String) -> Self {
        Self {
            tab_id,
            socket_path,
            current_size: (0, 0), // Will be set on first spawn/resize
            screen_cache: Arc::new(Mutex::new(None)),
            screen_requested_size: Arc::new(Mutex::new((0, 0))),
            last_screen_request_at: Arc::new(Mutex::new(Instant::now())),
            worker_running: Arc::new(AtomicBool::new(false)),
        }
    }

    fn send_msg_static(socket_path: &str, msg: &ClientMsg) -> Option<ServerMsg> {
        let mut stream = StdUnixStream::connect(socket_path).ok()?;

        let json = serde_json::to_vec(msg).ok()?;
        stream.write_all(&json).ok()?;
        stream.write_all(b"\n").ok()?;
        stream.flush().ok()?;

        // Read response
        let mut reader = BufReader::new(&stream);
        let mut line = String::new();
        reader.read_line(&mut line).ok()?;
        serde_json::from_str(&line).ok()
    }

    fn send_msg(&self, msg: ClientMsg) -> Option<ServerMsg> {
        Self::send_msg_static(&self.socket_path, &msg)
    }

    fn get_screen_sync(&self, rows: u16, cols: u16) -> Option<ScreenContent> {
        match self.send_msg(ClientMsg::GetScreen {
            tab_id: self.tab_id.clone(),
            rows,
            cols,
        }) {
            Some(ServerMsg::Screen { content, .. }) => Some(content),
            _ => None,
        }
    }

    // Send message without waiting for response (fire and forget)
    fn send_msg_no_response_static(socket_path: &str, msg: &ClientMsg) {
        if let Ok(mut stream) = StdUnixStream::connect(socket_path) {
            let json = serde_json::to_vec(&msg).unwrap();
            let _ = stream.write_all(&json);
            let _ = stream.write_all(b"\n");
            let _ = stream.flush();
        }
    }

    fn send_msg_no_response(&self, msg: ClientMsg) {
        Self::send_msg_no_response_static(&self.socket_path, &msg);
    }

    fn invalidate_screen_cache(&self) {
        if let Ok(mut cache) = self.screen_cache.lock() {
            *cache = None;
        }
    }

    fn start_screen_worker_if_needed(&self) {
        if self
            .worker_running
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            return;
        }

        let socket_path = self.socket_path.clone();
        let tab_id = self.tab_id.clone();
        let requested_size = self.screen_requested_size.clone();
        let last_screen_request_at = self.last_screen_request_at.clone();
        let cache = self.screen_cache.clone();
        let running = self.worker_running.clone();

        thread::spawn(move || {
            let mut last_error_log: Option<Instant> = None;
            let mut last_respawn_attempt: Option<Instant> = None;
            let mut stream: Option<StdUnixStream> = None;

            while running.load(Ordering::Relaxed) {
                let now = Instant::now();
                let since_last_request = match last_screen_request_at.lock() {
                    Ok(t) => now.duration_since(*t),
                    Err(_) => Duration::from_secs(10),
                };
                // This provider is not currently being rendered (background tab):
                // stop hitting daemon aggressively.
                if since_last_request > Duration::from_secs(2) {
                    // No demand for a while: tear down this worker entirely to reduce
                    // thread count when many tabs exist.
                    if since_last_request > Duration::from_secs(30) {
                        running.store(false, Ordering::Relaxed);
                        break;
                    }
                    thread::sleep(Duration::from_millis(500));
                    continue;
                }

                let (rows, cols) = match requested_size.lock() {
                    Ok(s) => *s,
                    Err(_) => {
                        thread::sleep(Duration::from_millis(40));
                        continue;
                    }
                };
                if rows == 0 || cols == 0 {
                    thread::sleep(Duration::from_millis(40));
                    continue;
                }

                let msg = ClientMsg::GetScreen {
                    tab_id: tab_id.clone(),
                    rows,
                    cols,
                };
                if stream.is_none() {
                    if let Ok(s) = StdUnixStream::connect(&socket_path) {
                        let _ = s.set_read_timeout(Some(Duration::from_millis(200)));
                        let _ = s.set_write_timeout(Some(Duration::from_millis(200)));
                        stream = Some(s);
                    }
                }
                let response = if let Some(s) = stream.as_mut() {
                    Self::send_msg_on_stream(s, &msg)
                } else {
                    None
                };

                if response.is_none() {
                    stream = None;
                }

                match response {
                    Some(ServerMsg::Screen { content, .. }) => {
                        if let Ok(mut c) = cache.lock() {
                            *c = Some(ScreenCacheEntry {
                                rows,
                                cols,
                                fetched_at: Instant::now(),
                                content,
                            });
                        }
                    }
                    Some(ServerMsg::Error { message }) => {
                        let now = Instant::now();
                        if message == "tab not found" {
                            let should_log = last_error_log
                                .map(|t| now.duration_since(t) >= Duration::from_secs(2))
                                .unwrap_or(true);
                            if should_log {
                                tracing::debug!("GetScreen miss for tab {}: {}", tab_id, message);
                                last_error_log = Some(now);
                            }
                            let should_respawn = last_respawn_attempt
                                .map(|t| now.duration_since(t) >= Duration::from_secs(1))
                                .unwrap_or(true);
                            if should_respawn {
                                last_respawn_attempt = Some(now);
                                let spawn = ClientMsg::Spawn {
                                    tab_id: tab_id.clone(),
                                    rows: rows.max(1),
                                    cols: cols.max(1),
                                };
                                Self::send_msg_no_response_static(&socket_path, &spawn);
                            }
                        } else {
                            let should_log = last_error_log
                                .map(|t| now.duration_since(t) >= Duration::from_secs(2))
                                .unwrap_or(true);
                            if should_log {
                                tracing::error!("GetScreen error for tab {}: {}", tab_id, message);
                                last_error_log = Some(now);
                            }
                        }
                    }
                    _ => {}
                }

                // Active tab (recent requests) gets smoother refresh; inactive gets lower load.
                if since_last_request <= Duration::from_millis(200) {
                    thread::sleep(Duration::from_millis(40));
                } else {
                    thread::sleep(Duration::from_millis(200));
                }
            }
            running.store(false, Ordering::Relaxed);
        });
    }
}

impl Drop for DaemonProvider {
    fn drop(&mut self) {
        self.worker_running.store(false, Ordering::Relaxed);
    }
}

impl TerminalProvider for DaemonProvider {
    fn spawn(&mut self, rows: u16, cols: u16) {
        self.current_size = (rows, cols); // Track size
        if let Ok(mut s) = self.screen_requested_size.lock() {
            *s = (rows, cols);
        }
        self.invalidate_screen_cache();
        self.send_msg(ClientMsg::Spawn {
            tab_id: self.tab_id.clone(),
            rows,
            cols,
        });
    }

    fn resize(&mut self, rows: u16, cols: u16) {
        // Skip if size hasn't changed
        if self.current_size == (rows, cols) {
            return;
        }

        self.current_size = (rows, cols);
        if let Ok(mut s) = self.screen_requested_size.lock() {
            *s = (rows, cols);
        }
        self.invalidate_screen_cache();

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

    fn paste(&mut self, text: &str) {
        self.send_msg_no_response(ClientMsg::Paste {
            tab_id: self.tab_id.clone(),
            data: text.to_string(),
        });
    }

    fn mouse_mode_enabled(&self) -> bool {
        match self.send_msg(ClientMsg::GetInputModes {
            tab_id: self.tab_id.clone(),
        }) {
            Some(ServerMsg::InputModes { mouse, .. }) => mouse,
            _ => false,
        }
    }

    fn bracketed_paste_enabled(&self) -> bool {
        match self.send_msg(ClientMsg::GetInputModes {
            tab_id: self.tab_id.clone(),
        }) {
            Some(ServerMsg::InputModes {
                bracketed_paste, ..
            }) => bracketed_paste,
            _ => false,
        }
    }

    fn get_screen(&self, rows: u16, cols: u16) -> ScreenContent {
        if let Ok(mut s) = self.screen_requested_size.lock() {
            *s = (rows, cols);
        }
        if let Ok(mut t) = self.last_screen_request_at.lock() {
            *t = Instant::now();
        }
        self.start_screen_worker_if_needed();

        if let Ok(cache) = self.screen_cache.lock() {
            if let Some(entry) = cache.as_ref() {
                if entry.rows == rows
                    && entry.cols == cols
                    && entry.fetched_at.elapsed() < Duration::from_secs(2)
                {
                    return entry.content.clone();
                }
                // Return most recent content while worker catches up to a new size.
                return entry.content.clone();
            }
        }

        // First-call fallback: do synchronous fetch to avoid a blank frame.
        // If daemon says tab is missing, synchronously spawn and retry once.
        match self.send_msg(ClientMsg::GetScreen {
            tab_id: self.tab_id.clone(),
            rows,
            cols,
        }) {
            Some(ServerMsg::Screen { content, .. }) => {
                if let Ok(mut cache) = self.screen_cache.lock() {
                    *cache = Some(ScreenCacheEntry {
                        rows,
                        cols,
                        fetched_at: Instant::now(),
                        content: content.clone(),
                    });
                }
                content
            }
            Some(ServerMsg::Error { message }) => {
                if message == "tab not found" {
                    let _ = self.send_msg(ClientMsg::Spawn {
                        tab_id: self.tab_id.clone(),
                        rows: self.current_size.0.max(1),
                        cols: self.current_size.1.max(1),
                    });
                    if let Some(content) = self.get_screen_sync(rows, cols) {
                        if let Ok(mut cache) = self.screen_cache.lock() {
                            *cache = Some(ScreenCacheEntry {
                                rows,
                                cols,
                                fetched_at: Instant::now(),
                                content: content.clone(),
                            });
                        }
                        return content;
                    }
                }
                if let Ok(cache) = self.screen_cache.lock() {
                    if let Some(entry) = cache.as_ref() {
                        return entry.content.clone();
                    }
                }
                ScreenContent::default()
            }
            _ => {
                if let Ok(cache) = self.screen_cache.lock() {
                    if let Some(entry) = cache.as_ref() {
                        return entry.content.clone();
                    }
                }
                ScreenContent::default()
            }
        }
    }

    fn scroll(&mut self, delta: i32) {
        self.send_msg_no_response(ClientMsg::Scroll {
            tab_id: self.tab_id.clone(),
            delta,
        });
    }
}
