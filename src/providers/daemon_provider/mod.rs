mod worker;

use crate::protocol::{ClientMsg, ServerMsg};
use crate::terminal_provider::{ScreenContent, TerminalProvider};
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream as StdUnixStream;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct ScreenCacheEntry {
    content: ScreenContent,
    rows: u16,
    cols: u16,
}

pub struct DaemonProvider {
    tab_id: String,
    socket_path: String,
    current_size: (u16, u16),
    screen_cache: Arc<Mutex<Option<ScreenCacheEntry>>>,
    screen_requested_size: Arc<Mutex<(u16, u16)>>,
    last_screen_request_at: Arc<Mutex<Instant>>,
    worker_running: Arc<AtomicBool>,
    write_stream: Option<StdUnixStream>,
    /// Channel to send messages to worker for writing on subscribe connection
    worker_tx: Arc<Mutex<Option<mpsc::Sender<Vec<u8>>>>>,
    screen_generation: Arc<AtomicU64>,
}

impl DaemonProvider {
    /// Read a response from a stream. Supports binary frames (0x00 prefix) and JSON lines.
    /// Returns Ok(Some(msg)) on data, Ok(None) on timeout, Err on disconnect/error.
    fn read_response(
        stream: &mut StdUnixStream,
        buf: &mut Vec<u8>,
    ) -> Result<Option<ServerMsg>, ()> {
        let mut first = [0u8; 1];
        match std::io::Read::read_exact(stream, &mut first) {
            Ok(()) => {}
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                return Ok(None); // Timeout
            }
            Err(_) => return Err(()), // Real disconnect
        }

        if first[0] == 0x00 {
            let mut len_bytes = [0u8; 4];
            if std::io::Read::read_exact(stream, &mut len_bytes).is_err() {
                return Err(());
            }
            let len = u32::from_le_bytes(len_bytes) as usize;
            buf.clear();
            buf.resize(len, 0);
            if std::io::Read::read_exact(stream, buf).is_err() {
                return Err(());
            }
            Ok(rmp_serde::from_slice(buf).ok())
        } else {
            buf.clear();
            buf.push(first[0]);
            let mut chunk = [0u8; 8192];
            loop {
                match std::io::Read::read(stream, &mut chunk) {
                    Ok(0) => return Err(()),
                    Ok(n) => {
                        if let Some(nl) = chunk[..n].iter().position(|&b| b == b'\n') {
                            buf.extend_from_slice(&chunk[..nl]);
                            break;
                        }
                        buf.extend_from_slice(&chunk[..n]);
                    }
                    Err(_) => return Err(()),
                }
            }
            Ok(serde_json::from_slice(buf).ok())
        }
    }

    /// Non-blocking check if socket has data ready to read via poll(2).
    fn socket_readable(stream: &StdUnixStream) -> bool {
        let fd = stream.as_raw_fd();
        let mut pfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        unsafe { libc::poll(&mut pfd, 1, 0) > 0 && (pfd.revents & libc::POLLIN) != 0 }
    }

    #[allow(dead_code)]
    fn send_msg_on_stream(
        stream: &mut StdUnixStream,
        msg: &ClientMsg,
        read_buf: &mut Vec<u8>,
    ) -> Option<ServerMsg> {
        let json = serde_json::to_vec(msg).ok()?;
        stream.write_all(&json).ok()?;
        stream.write_all(b"\n").ok()?;
        stream.flush().ok()?;
        Self::read_response(stream, read_buf).ok().flatten()
    }

    pub fn new(tab_id: String, socket_path: String) -> Self {
        Self {
            tab_id,
            socket_path,
            current_size: (0, 0),
            screen_cache: Arc::new(Mutex::new(None)),
            screen_requested_size: Arc::new(Mutex::new((0, 0))),
            last_screen_request_at: Arc::new(Mutex::new(Instant::now())),
            worker_running: Arc::new(AtomicBool::new(false)),
            write_stream: None,
            worker_tx: Arc::new(Mutex::new(None)),
            screen_generation: Arc::new(AtomicU64::new(0)),
        }
    }

    fn send_msg_static(socket_path: &str, msg: &ClientMsg) -> Option<ServerMsg> {
        let mut stream = StdUnixStream::connect(socket_path).ok()?;
        let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
        let _ = stream.set_write_timeout(Some(Duration::from_millis(200)));

        let json = serde_json::to_vec(msg).ok()?;
        stream.write_all(&json).ok()?;
        stream.write_all(b"\n").ok()?;
        stream.flush().ok()?;

        let mut buf = Vec::with_capacity(64 * 1024);
        Self::read_response(&mut stream, &mut buf).ok().flatten()
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
            if let Ok(json) = serde_json::to_vec(&msg) {
                let _ = stream.write_all(&json);
                let _ = stream.write_all(b"\n");
                let _ = stream.flush();
            }
        }
    }

    /// Send fire-and-forget message over the persistent write connection.
    /// Uses binary (MessagePack) framing for minimal serialization overhead.
    fn send_msg_no_response(&mut self, msg: ClientMsg) {
        let bin = match rmp_serde::to_vec(&msg) {
            Ok(b) => b,
            Err(_) => return,
        };
        let len = (bin.len() as u32).to_le_bytes();

        // Try writing on the existing stream
        if let Some(ref mut stream) = self.write_stream {
            if stream.write_all(&[0x00]).is_ok()
                && stream.write_all(&len).is_ok()
                && stream.write_all(&bin).is_ok()
                && stream.flush().is_ok()
            {
                return;
            }
            self.write_stream = None;
        }

        // (Re)connect
        if let Ok(stream) = StdUnixStream::connect(&self.socket_path) {
            let _ = stream.set_write_timeout(Some(Duration::from_millis(100)));
            self.write_stream = Some(stream);
            if let Some(ref mut s) = self.write_stream {
                let _ = s.write_all(&[0x00]);
                let _ = s.write_all(&len);
                let _ = s.write_all(&bin);
                let _ = s.flush();
            }
        }
    }

    fn invalidate_screen_cache(&self) {
        if let Ok(mut cache) = self.screen_cache.lock() {
            *cache = None;
        }
    }

    fn cache_screen(&self, content: ScreenContent, rows: u16, cols: u16) {
        if let Ok(mut cache) = self.screen_cache.lock() {
            *cache = Some(ScreenCacheEntry {
                content,
                rows,
                cols,
            });
        }
    }

    fn cached_screen(&self, rows: u16, cols: u16) -> Option<ScreenContent> {
        if let Ok(cache) = self.screen_cache.lock() {
            if let Some(entry) = cache.as_ref() {
                if entry.rows == rows && entry.cols == cols {
                    return Some(entry.content.clone());
                }
            }
        }
        None
    }

    fn try_send_via_worker_channel(&self, msg: &ClientMsg) -> bool {
        let Ok(wtx) = self.worker_tx.lock() else {
            return false;
        };
        let Some(tx) = &*wtx else {
            return false;
        };
        let Ok(bin) = rmp_serde::to_vec(msg) else {
            return false;
        };
        let mut frame = Vec::with_capacity(5 + bin.len());
        frame.push(0x00);
        frame.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        frame.extend_from_slice(&bin);
        tx.send(frame).is_ok()
    }

}


impl Drop for DaemonProvider {
    fn drop(&mut self) {
        self.worker_running.store(false, Ordering::Relaxed);
    }
}

impl TerminalProvider for DaemonProvider {
    fn spawn(&mut self, rows: u16, cols: u16) {
        self.current_size = (rows, cols);
        if let Ok(mut s) = self.screen_requested_size.lock() {
            *s = (rows, cols);
        }
        // Mark as active so worker reconnects immediately instead of sleeping
        if let Ok(mut t) = self.last_screen_request_at.lock() {
            *t = Instant::now();
        }
        self.invalidate_screen_cache();
        // Send Spawn FIRST (sync — waits for daemon to register the tab),
        // THEN start the worker. This prevents the worker's Subscribe from
        // racing ahead and getting "tab not found" before Spawn completes.
        let _resp = self.send_msg(ClientMsg::Spawn {
            tab_id: self.tab_id.clone(),
            rows,
            cols,
            cwd: None,
            shell: None,
            env: None,
        });
        self.start_screen_worker_if_needed();
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
        if let Ok(mut t) = self.last_screen_request_at.lock() {
            *t = Instant::now();
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
        let msg = ClientMsg::Input {
            tab_id: self.tab_id.clone(),
            data: bytes.to_vec(),
        };
        // Try worker channel first (same connection as subscribe = faster echo).
        if self.try_send_via_worker_channel(&msg) {
            return;
        }
        self.send_msg_no_response(msg);
    }

    fn paste(&mut self, text: &str) {
        let msg = ClientMsg::Paste {
            tab_id: self.tab_id.clone(),
            data: text.to_string(),
        };
        if self.try_send_via_worker_channel(&msg) {
            return;
        }
        self.send_msg_no_response(msg);
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

        if let Some(content) = self.cached_screen(rows, cols) {
            // Consume bell: clear it in cache so it only fires once per BEL.
            if content.bell {
                if let Ok(mut c) = self.screen_cache.lock() {
                    if let Some(ref mut entry) = *c {
                        entry.content.bell = false;
                    }
                }
            }
            return content;
        }

        // First-call fallback: do synchronous fetch to avoid a blank frame.
        // If daemon says tab is missing, synchronously spawn and retry once.
        match self.send_msg(ClientMsg::GetScreen {
            tab_id: self.tab_id.clone(),
            rows,
            cols,
        }) {
            Some(ServerMsg::Screen { content, .. }) => {
                self.screen_generation.fetch_add(1, Ordering::Relaxed);
                self.cache_screen(content.clone(), rows, cols);
                content
            }
            Some(ServerMsg::Error { message }) => {
                if message == "tab not found" {
                    let _ = self.send_msg(ClientMsg::Spawn {
                        tab_id: self.tab_id.clone(),
                        rows: rows.max(1),
                        cols: cols.max(1),
                        cwd: None,
                        shell: None,
                        env: None,
                    });
                    if let Some(content) = self.get_screen_sync(rows, cols) {
                        self.screen_generation.fetch_add(1, Ordering::Relaxed);
                        self.cache_screen(content.clone(), rows, cols);
                        return content;
                    }
                }
                if let Some(content) = self.cached_screen(rows, cols) {
                    return content;
                }
                ScreenContent::default()
            }
            _ => {
                if let Some(content) = self.cached_screen(rows, cols) {
                    return content;
                }
                ScreenContent::default()
            }
        }
    }

    fn scroll(&mut self, delta: i32) {
        Self::send_msg_no_response_static(
            &self.socket_path,
            &ClientMsg::Scroll {
                tab_id: self.tab_id.clone(),
                delta,
            },
        );
        // Ensure scroll feedback is visible immediately in copy-mode interactions.
        let (rows, cols) = self
            .screen_requested_size
            .lock()
            .map(|s| *s)
            .unwrap_or(self.current_size);
        let rows = rows.max(1);
        let cols = cols.max(1);
        if let Some(content) = self.get_screen_sync(rows, cols) {
            self.screen_generation.fetch_add(1, Ordering::Relaxed);
            self.cache_screen(content, rows, cols);
        }
    }

    fn screen_generation(&self) -> u64 {
        self.screen_generation.load(Ordering::Relaxed)
    }

    fn focus_events_enabled(&self) -> bool {
        if let Ok(cache) = self.screen_cache.lock() {
            if let Some(ref entry) = *cache {
                return entry.content.focus_events_enabled;
            }
        }
        false
    }
}
