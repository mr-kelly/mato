use crate::protocol::{ClientMsg, ServerMsg};
use crate::terminal_provider::{ScreenContent, TerminalProvider};
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream as StdUnixStream;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
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
            let json = serde_json::to_vec(&msg).unwrap();
            let _ = stream.write_all(&json);
            let _ = stream.write_all(b"\n");
            let _ = stream.flush();
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
        if let Ok(mut cache) = self.screen_cache.lock() {
            if let Some(entry) = cache.as_mut() {
                if entry.rows == rows && entry.cols == cols {
                    let content = entry.content.clone();
                    // Clear bell flag after reading so it only fires once
                    if content.bell {
                        entry.content.bell = false;
                    }
                    return Some(content);
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

    fn start_screen_worker_if_needed(&self) {
        if self
            .worker_running
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            return;
        }

        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        if let Ok(mut wtx) = self.worker_tx.lock() {
            *wtx = Some(tx);
        }

        let socket_path = self.socket_path.clone();
        let tab_id = self.tab_id.clone();
        let requested_size = self.screen_requested_size.clone();
        let last_screen_request_at = self.last_screen_request_at.clone();
        let cache = self.screen_cache.clone();
        let running = self.worker_running.clone();
        let screen_gen = self.screen_generation.clone();

        thread::spawn(move || {
            let mut last_error_log: Option<Instant> = None;

            let mut stream: Option<StdUnixStream> = None;
            let mut read_buf: Vec<u8> = Vec::with_capacity(256 * 1024);
            let mut subscribed_size: (u16, u16) = (0, 0);

            while running.load(Ordering::Relaxed) {
                let now = Instant::now();
                let since_last_request = match last_screen_request_at.lock() {
                    Ok(t) => now.duration_since(*t),
                    Err(_) => Duration::from_secs(10),
                };
                // Background tab: slow down or stop
                if since_last_request > Duration::from_secs(2) {
                    if since_last_request > Duration::from_secs(30) {
                        running.store(false, Ordering::Relaxed);
                        break;
                    }
                    stream = None; // drop subscription connection
                    subscribed_size = (0, 0);
                    thread::sleep(Duration::from_millis(50));
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

                // Connect and subscribe if needed
                if stream.is_none() {
                    tracing::debug!(
                        "[worker] tab={} connecting to subscribe (rows={} cols={})",
                        tab_id,
                        rows,
                        cols
                    );
                    if let Ok(s) = StdUnixStream::connect(&socket_path) {
                        // Safety timeout for partial reads; poll() avoids blocking normally
                        let _ = s.set_read_timeout(Some(Duration::from_millis(5)));
                        let _ = s.set_write_timeout(Some(Duration::from_millis(100)));
                        // Send Subscribe message
                        let sub_msg = ClientMsg::Subscribe {
                            tab_id: tab_id.clone(),
                            rows,
                            cols,
                        };
                        if let Ok(json) = serde_json::to_vec(&sub_msg) {
                            let mut s = s;
                            if s.write_all(&json).is_ok()
                                && s.write_all(b"\n").is_ok()
                                && s.flush().is_ok()
                            {
                                subscribed_size = (rows, cols);
                                stream = Some(s);
                                tracing::debug!("[worker] tab={} subscribe sent OK", tab_id);
                            } else {
                                tracing::warn!("[worker] tab={} subscribe write failed", tab_id);
                            }
                        }
                    } else {
                        tracing::debug!("[worker] tab={} connect failed", tab_id);
                    }
                    if stream.is_none() {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                }

                // If size changed, send Resize on the subscription connection
                if subscribed_size != (rows, cols) {
                    if let Some(ref mut s) = stream {
                        let resize_msg = ClientMsg::Resize {
                            tab_id: tab_id.clone(),
                            rows,
                            cols,
                        };
                        if let Ok(bin) = rmp_serde::to_vec(&resize_msg) {
                            let len = (bin.len() as u32).to_le_bytes();
                            if s.write_all(&[0x00]).is_ok()
                                && s.write_all(&len).is_ok()
                                && s.write_all(&bin).is_ok()
                                && s.flush().is_ok()
                            {
                                subscribed_size = (rows, cols);
                            }
                        }
                    }
                }

                // Drain pending outgoing messages (Input/Paste) from channel
                // and write them on the subscribe stream for single-connection echo
                if let Some(ref mut s) = stream {
                    let mut wrote = false;
                    while let Ok(frame) = rx.try_recv() {
                        let _ = s.write_all(&frame);
                        wrote = true;
                    }
                    if wrote {
                        let _ = s.flush();
                    }
                }

                // Read pushed screen update — use poll() to avoid blocking when no data
                let readable = stream.as_ref().is_some_and(Self::socket_readable);
                let response = if readable {
                    if let Some(ref mut s) = stream {
                        match Self::read_response(s, &mut read_buf) {
                            Ok(msg) => {
                                // After read, drain any messages that arrived during the read
                                let mut wrote = false;
                                while let Ok(frame) = rx.try_recv() {
                                    let _ = s.write_all(&frame);
                                    wrote = true;
                                }
                                if wrote {
                                    let _ = s.flush();
                                }
                                msg
                            }
                            Err(()) => {
                                stream = None;
                                subscribed_size = (0, 0);
                                thread::sleep(Duration::from_millis(100));
                                continue;
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    // No data on socket — brief sleep to keep drain cycle fast
                    thread::sleep(Duration::from_micros(200));
                    None
                };

                match response {
                    Some(ServerMsg::Screen { content, .. }) => {
                        screen_gen.fetch_add(1, Ordering::Relaxed);
                        if let Ok(mut c) = cache.lock() {
                            *c = Some(ScreenCacheEntry {
                                content,
                                rows: subscribed_size.0,
                                cols: subscribed_size.1,
                            });
                        }
                    }
                    Some(ServerMsg::ScreenDiff {
                        changed_lines,
                        cursor,
                        cursor_shape,
                        title,
                        bell,
                    }) => {
                        screen_gen.fetch_add(1, Ordering::Relaxed);
                        if let Ok(mut c) = cache.lock() {
                            if let Some(ref mut entry) = *c {
                                // Apply diff to cached screen
                                for (line_idx, new_line) in changed_lines {
                                    let idx = line_idx as usize;
                                    if idx < entry.content.lines.len() {
                                        entry.content.lines[idx] = new_line;
                                    }
                                }
                                entry.content.cursor = cursor;
                                entry.content.cursor_shape = cursor_shape;
                                entry.content.title = title;
                                entry.content.bell = bell;
                            }
                        }
                    }
                    Some(ServerMsg::Error { message }) => {
                        let now = Instant::now();
                        if message == "tab not found" {
                            // Worker must NOT spawn the tab — that's the main thread's job (spawn()).
                            // If we spawn here with rows=1,cols=1 we race and create a 1x1 terminal.
                            // Just wait and retry Subscribe; the main thread will have called spawn().
                            let should_log = last_error_log
                                .map(|t| now.duration_since(t) >= Duration::from_secs(2))
                                .unwrap_or(true);
                            if should_log {
                                tracing::debug!(
                                    "[worker] tab={} 'tab not found', retrying subscribe (main thread handles spawn)",
                                    tab_id
                                );
                                last_error_log = Some(now);
                            }
                            // Reconnect after error
                            stream = None;
                            subscribed_size = (0, 0);
                            thread::sleep(Duration::from_millis(100));
                        } else {
                            let should_log = last_error_log
                                .map(|t| now.duration_since(t) >= Duration::from_secs(2))
                                .unwrap_or(true);
                            if should_log {
                                tracing::error!("Push error for tab {}: {}", tab_id, message);
                                last_error_log = Some(now);
                            }
                        }
                    }
                    None => {
                        // Timeout — normal in push mode, just wait for next push.
                        // But if we've never received data on this subscription,
                        // something may be wrong — reconnect after extended silence.
                    }
                    _ => {}
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
}
