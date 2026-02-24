use crate::protocol::{ClientMsg, ServerMsg};
use std::io::Write;
use std::os::unix::net::UnixStream as StdUnixStream;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use super::{DaemonProvider, ScreenCacheEntry};

impl DaemonProvider {
    pub(super) fn start_screen_worker_if_needed(&self) {
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
        let pending_graphics = self.pending_graphics.clone();
        let cached_cwd = self.cached_cwd.clone();

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
                        // Update cwd from screen content
                        if let Some(ref cwd) = content.cwd {
                            if let Ok(mut c) = cached_cwd.lock() {
                                *c = Some(cwd.clone());
                            }
                        }
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
                        focus_events_enabled,
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
                                entry.content.focus_events_enabled = focus_events_enabled;
                            }
                        }
                    }
                    Some(ServerMsg::Graphics { payloads, .. }) => {
                        // Buffer APC sequences for the client to re-emit to outer terminal
                        if let Ok(mut g) = pending_graphics.lock() {
                            g.extend(payloads);
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
