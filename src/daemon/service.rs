use crate::config::Config;
use crate::daemon::signals::SignalHandler;
use crate::protocol::{ClientMsg, ServerMsg};
use crate::providers::PtyProvider;
use crate::terminal_provider::TerminalProvider;
use anyhow::Result;
use dashmap::DashMap;
use parking_lot::Mutex;
use semver::Version;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const VERSION_URL: &str = "https://mato.sh/version.txt";

fn parse_semver_like(input: &str) -> Option<Version> {
    let s = input.trim().strip_prefix('v').unwrap_or(input.trim());
    Version::parse(s).ok()
}

fn compute_update_available(remote_raw: &str, current_raw: &str) -> Option<String> {
    let remote = remote_raw.trim();
    if remote.is_empty() {
        return None;
    }
    match (parse_semver_like(remote), parse_semver_like(current_raw)) {
        (Some(remote_v), Some(current_v)) if remote_v > current_v => Some(remote.to_string()),
        _ => None,
    }
}

fn is_disconnect_error(err: &anyhow::Error) -> bool {
    if let Some(ioe) = err.downcast_ref::<std::io::Error>() {
        matches!(
            ioe.kind(),
            std::io::ErrorKind::BrokenPipe
                | std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::ConnectionAborted
                | std::io::ErrorKind::UnexpectedEof
        )
    } else {
        false
    }
}

pub struct Daemon {
    tabs: Arc<DashMap<String, Arc<Mutex<PtyProvider>>>>,
    signals: SignalHandler,
    config: Arc<Mutex<Config>>,
    client_count: Arc<AtomicUsize>,
    /// None = up to date or check failed; Some(ver) = update available
    latest_version: Arc<Mutex<Option<String>>>,
}

impl Daemon {
    pub fn new() -> Self {
        let config = Config::load();
        tracing::info!("Loaded config: emulator={}", config.emulator);

        Self {
            tabs: Arc::new(DashMap::new()),
            signals: SignalHandler::new(),
            config: Arc::new(Mutex::new(config)),
            client_count: Arc::new(AtomicUsize::new(0)),
            latest_version: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn run(&self, socket_path: &str) -> Result<()> {
        let _ = std::fs::remove_file(socket_path);
        let listener = UnixListener::bind(socket_path)?;

        // Set socket permissions to 0700 (owner only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o700);
            std::fs::set_permissions(socket_path, perms)?;
            tracing::info!("Socket permissions set to 0700");
        }

        tracing::info!("Daemon listening on {}", socket_path);
        tracing::info!("Active tabs at startup: {}", self.tabs.len());

        // Background update checker: fetch every hour
        {
            let latest_version = self.latest_version.clone();
            tokio::spawn(async move {
                loop {
                    match reqwest::get(VERSION_URL).await {
                        Ok(resp) if resp.status().is_success() => {
                            if let Ok(text) = resp.text().await {
                                let update = compute_update_available(&text, CURRENT_VERSION);
                                *latest_version.lock() = update;
                            }
                        }
                        _ => {
                            // fetch failed: log and leave state unchanged
                            tracing::debug!("Update check failed: could not reach {}", VERSION_URL);
                        }
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                }
            });
        }

        loop {
            // Check for shutdown signal
            if self.signals.should_shutdown() {
                tracing::info!("Received shutdown signal, exiting gracefully");
                self.shutdown();
                break;
            }

            // Check for reload signal
            if self.signals.should_reload() {
                self.reload_config();
            }

            // Accept connections with timeout
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _)) => {
                            let client_id = self.client_count.fetch_add(1, Ordering::Relaxed) + 1;
                            tracing::debug!("Client #{} connecting (total: {})", client_id, client_id);

                            let tabs = self.tabs.clone();
                            let config = self.config.clone();
                            let client_count = self.client_count.clone();
                            let latest_version = self.latest_version.clone();

                            tokio::spawn(async move {
                                if let Err(e) = handle_client(stream, tabs, config, client_id, latest_version).await {
                                    if is_disconnect_error(&e) {
                                        tracing::debug!("Client #{} disconnected during IO: {}", client_id, e);
                                    } else {
                                        tracing::error!("Client #{} error: {}", client_id, e);
                                    }
                                }
                                let remaining = client_count.fetch_sub(1, Ordering::Relaxed) - 1;
                                tracing::debug!("Client #{} disconnected (remaining: {})", client_id, remaining);
                            });
                        }
                        Err(e) => {
                            tracing::error!("Accept error: {}", e);
                        }
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                    // Check signals periodically
                }
            }
        }

        Ok(())
    }

    fn reload_config(&self) {
        tracing::info!("Received SIGHUP, reloading configuration");
        let new_config = Config::load();
        let mut config = self.config.lock();
        *config = new_config;
        tracing::info!("Configuration reloaded: emulator={}", config.emulator);
    }

    fn shutdown(&self) {
        tracing::info!("Starting graceful shutdown");
        tracing::info!("Closing {} active tabs", self.tabs.len());
        self.tabs.clear();
        tracing::info!("Graceful shutdown complete");
    }
}

impl Default for Daemon {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::compute_update_available;

    #[test]
    fn update_none_when_same_version() {
        assert_eq!(compute_update_available("0.6.0", "0.6.0"), None);
    }

    #[test]
    fn update_when_remote_newer_patch() {
        assert_eq!(
            compute_update_available("0.6.1", "0.6.0"),
            Some("0.6.1".to_string())
        );
    }

    #[test]
    fn no_update_when_remote_older() {
        assert_eq!(compute_update_available("0.5.9", "0.6.0"), None);
    }

    #[test]
    fn update_from_prerelease_to_stable() {
        assert_eq!(
            compute_update_available("0.6.0", "0.6.0-alpha.1"),
            Some("0.6.0".to_string())
        );
    }

    #[test]
    fn no_update_when_remote_prerelease_below_current_stable() {
        assert_eq!(compute_update_available("0.6.0-alpha.2", "0.6.0"), None);
    }

    #[test]
    fn accepts_v_prefix() {
        assert_eq!(
            compute_update_available("v0.6.1", "0.6.0"),
            Some("v0.6.1".to_string())
        );
    }

    #[test]
    fn no_update_on_invalid_remote_text() {
        assert_eq!(compute_update_available("latest", "0.6.0"), None);
    }
}

pub async fn handle_client(
    stream: UnixStream,
    tabs: Arc<DashMap<String, Arc<Mutex<PtyProvider>>>>,
    _config: Arc<Mutex<Config>>,
    client_id: usize,
    latest_version: Arc<Mutex<Option<String>>>,
) -> Result<()> {
    tracing::debug!("Client #{} handler started", client_id);
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut bin_buf: Vec<u8> = Vec::with_capacity(4096);
    let mut last_screen_hash: u64 = 0;

    loop {
        // Read first byte to detect binary (0x00) vs JSON
        let msg: ClientMsg = {
            let buf = reader.fill_buf().await?;
            if buf.is_empty() {
                tracing::debug!("Client #{} disconnected", client_id);
                break;
            }
            if buf[0] == 0x00 {
                // Binary frame: 0x00 + 4-byte LE length + MessagePack payload
                reader.consume(1);
                let len = reader.read_u32_le().await? as usize;
                let mut payload = vec![0u8; len];
                reader.read_exact(&mut payload).await?;
                match rmp_serde::from_slice(&payload) {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::error!("Failed to parse binary message: {}", e);
                        continue;
                    }
                }
            } else {
                // JSON line
                line.clear();
                let n = reader.read_line(&mut line).await?;
                if n == 0 {
                    tracing::debug!("Client #{} disconnected", client_id);
                    break;
                }
                match serde_json::from_str(&line) {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::error!("Failed to parse message: {} | Line: {}", e, line.trim());
                        continue;
                    }
                }
            }
        };

        let response = match msg {
            ClientMsg::Hello { .. } => ServerMsg::Welcome {
                version: CURRENT_VERSION.into(),
            },

            ClientMsg::Spawn {
                tab_id,
                rows,
                cols,
                cwd,
                shell,
                env,
            } => {
                if tabs.contains_key(&tab_id) {
                    tracing::debug!(
                        "[daemon] Spawn tab={} already exists (total tabs: {})",
                        tab_id,
                        tabs.len()
                    );
                    // Don't resize on reconnect - it would clear the screen
                    // Client will send explicit Resize if needed
                    ServerMsg::Welcome {
                        version: "already exists".into(),
                    }
                } else {
                    tracing::debug!(
                        "[daemon] Spawn NEW tab={} ({}x{}) total_before={}",
                        tab_id,
                        rows,
                        cols,
                        tabs.len()
                    );
                    // Insert BEFORE spawn_with_options so concurrent Subscribe
                    // requests can find the tab immediately (not after shell fork).
                    let provider = Arc::new(Mutex::new(PtyProvider::new()));
                    tabs.insert(tab_id.clone(), provider.clone());
                    tracing::debug!(
                        "[daemon] Spawn tab={} inserted, total tabs now: {}",
                        tab_id,
                        tabs.len()
                    );
                    {
                        let mut p = provider.lock();
                        p.spawn_with_options(
                            rows,
                            cols,
                            cwd.as_deref(),
                            shell.as_deref(),
                            env.as_deref(),
                        );
                    }
                    ServerMsg::Welcome {
                        version: "spawned".into(),
                    }
                }
            }

            ClientMsg::Input { tab_id, data } => {
                if let Some(tab) = tabs.get(&tab_id) {
                    let mut tab = tab.lock();
                    tab.ensure_running();
                    tab.write(&data);
                }
                continue;
            }

            ClientMsg::Paste { tab_id, data } => {
                if let Some(tab) = tabs.get(&tab_id) {
                    let mut tab = tab.lock();
                    tab.ensure_running();
                    tab.paste(&data);
                }
                continue;
            }

            ClientMsg::GetInputModes { tab_id } => {
                if let Some(tab) = tabs.get(&tab_id) {
                    let mut tab = tab.lock();
                    tab.ensure_running();
                    ServerMsg::InputModes {
                        mouse: tab.mouse_mode_enabled(),
                        bracketed_paste: tab.bracketed_paste_enabled(),
                    }
                } else {
                    ServerMsg::Error {
                        message: "tab not found".into(),
                    }
                }
            }

            ClientMsg::Resize { tab_id, rows, cols } => {
                let strategy = _config.lock().resize_strategy.clone();
                if strategy == crate::config::ResizeStrategy::Sync {
                    if let Some(tab) = tabs.get(&tab_id) {
                        let mut tab = tab.lock();
                        tab.resize(rows.max(1), cols.max(1));
                        tracing::debug!("Resized tab {} to {}x{} (sync mode)", tab_id, rows, cols);
                    }
                } else {
                    tracing::debug!(
                        "Ignoring resize for tab {} ({}x{}) - fixed mode",
                        tab_id,
                        rows,
                        cols
                    );
                }
                continue;
            }

            ClientMsg::GetScreen { tab_id, rows, cols } => {
                if let Some(tab) = tabs.get(&tab_id) {
                    let (bin, hash) = {
                        let mut tab = tab.lock();
                        tab.spawn(rows.max(1), cols.max(1));
                        let content = tab.get_screen(rows, cols);
                        let response = ServerMsg::Screen { tab_id, content };
                        let bin = rmp_serde::to_vec(&response).unwrap_or_default();
                        let hash = {
                            use std::hash::{Hash, Hasher};
                            let mut h = std::collections::hash_map::DefaultHasher::new();
                            bin.hash(&mut h);
                            h.finish()
                        };
                        (bin, hash)
                    }; // MutexGuard dropped here, before any .await
                    if hash == last_screen_hash {
                        let json = serde_json::to_vec(&ServerMsg::ScreenUnchanged)?;
                        writer.write_all(&json).await?;
                        writer.write_all(b"\n").await?;
                    } else {
                        last_screen_hash = hash;
                        let len = (bin.len() as u32).to_le_bytes();
                        writer.write_all(&[0x00]).await?;
                        writer.write_all(&len).await?;
                        writer.write_all(&bin).await?;
                    }
                    writer.flush().await?;
                    continue;
                } else {
                    tracing::debug!("Tab not found: {}", tab_id);
                    ServerMsg::Error {
                        message: "tab not found".into(),
                    }
                }
            }

            ClientMsg::GetIdleStatus => {
                let now = Instant::now();
                let idle: Vec<(String, u64)> = tabs
                    .iter()
                    .map(|entry| {
                        let secs = now
                            .duration_since(*entry.value().lock().last_output.lock().unwrap())
                            .as_secs();
                        (entry.key().clone(), secs)
                    })
                    .collect();
                ServerMsg::IdleStatus { tabs: idle }
            }

            ClientMsg::GetProcessStatus => {
                let procs: Vec<(String, u32)> = tabs
                    .iter()
                    .filter_map(|entry| {
                        let pid = entry.value().lock().child_pid()?;
                        Some((entry.key().clone(), pid))
                    })
                    .collect();
                ServerMsg::ProcessStatus { tabs: procs }
            }

            ClientMsg::GetUpdateStatus => {
                let latest = latest_version.lock().clone();
                ServerMsg::UpdateStatus { latest }
            }

            ClientMsg::ClosePty { tab_id } => {
                if let Some((_, entry)) = tabs.remove(&tab_id) {
                    tracing::info!("Closing PTY for tab {}", tab_id);
                    drop(entry);
                    continue;
                } else {
                    tracing::debug!("Attempted to close non-existent tab {}", tab_id);
                    continue;
                }
            }
            ClientMsg::Scroll { tab_id, delta } => {
                if let Some(entry) = tabs.get(&tab_id) {
                    let mut tab = entry.lock();
                    tab.ensure_running();
                    tab.scroll(delta);
                }
                continue;
            }

            ClientMsg::Subscribe { tab_id, rows, cols } => {
                // Enter push mode: continuously push screen updates on PTY output.
                // This takes over the connection — no more request/response.
                let notify = if let Some(entry) = tabs.get(&tab_id) {
                    let mut tab = entry.lock();
                    tab.spawn(rows.max(1), cols.max(1));
                    let n = tab.output_notify.clone();
                    drop(tab);
                    Some(n)
                } else {
                    None
                };
                let Some(notify) = notify else {
                    tracing::debug!("[daemon] Subscribe tab={} not found", tab_id);
                    let json = serde_json::to_vec(&ServerMsg::Error {
                        message: "tab not found".into(),
                    })?;
                    writer.write_all(&json).await?;
                    writer.write_all(b"\n").await?;
                    writer.flush().await?;
                    continue;
                };
                tracing::debug!("[daemon] Subscribe tab={} found", tab_id);

                tracing::debug!(
                    "Client #{} subscribed to tab {} (push mode)",
                    client_id,
                    tab_id
                );
                let mut sub_rows = rows;
                let mut sub_cols = cols;
                line.clear();
                let mut last_sent_screen: Option<crate::terminal_provider::ScreenContent> = None;

                // Send initial screen immediately so client doesn't wait
                if let Some(entry) = tabs.get(&tab_id) {
                    let content = {
                        let tab = entry.lock();
                        tab.get_screen(sub_rows, sub_cols)
                    };
                    let response = ServerMsg::Screen {
                        tab_id: tab_id.clone(),
                        content: content.clone(),
                    };
                    let bin = rmp_serde::to_vec(&response).unwrap_or_default();
                    last_sent_screen = Some(content);
                    let len = (bin.len() as u32).to_le_bytes();
                    let mut frame = Vec::with_capacity(5 + bin.len());
                    frame.push(0x00);
                    frame.extend_from_slice(&len);
                    frame.extend_from_slice(&bin);
                    let _ = writer.write_all(&frame).await;
                    let _ = writer.flush().await;
                }

                // Push loop: wait for PTY output, then send screen
                let mut skip_coalesce = false;
                let mut push_frame_buf: Vec<u8> = Vec::with_capacity(64 * 1024);
                loop {
                    // Wait for output or timeout (200ms max to catch missed notifies)
                    tokio::select! {
                        _ = notify.notified() => {}
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(200)) => {}
                        // Also check for incoming messages (binary or JSON)
                        result = async {
                            let buf = reader.fill_buf().await?;
                            if buf.is_empty() {
                                return Ok::<Option<ClientMsg>, anyhow::Error>(None); // disconnected
                            }
                            if buf[0] == 0x00 {
                                reader.consume(1);
                                let len = reader.read_u32_le().await? as usize;
                                bin_buf.clear();
                                bin_buf.resize(len, 0);
                                reader.read_exact(&mut bin_buf).await?;
                                Ok(rmp_serde::from_slice(&bin_buf).ok())
                            } else {
                                line.clear();
                                let n = reader.read_line(&mut line).await?;
                                if n == 0 { return Ok(None); }
                                Ok(serde_json::from_str(&line).ok())
                            }
                        } => {
                            match result {
                                Ok(None) => break, // disconnected
                                Ok(Some(msg)) => {
                                    match msg {
                                        ClientMsg::Resize { rows: r, cols: c, .. } => {
                                            sub_rows = r;
                                            sub_cols = c;
                                            last_sent_screen = None; // Force full screen after resize
                                            if let Some(entry) = tabs.get(&tab_id) {
                                                let config = _config.lock();
                                                if matches!(config.resize_strategy, crate::config::ResizeStrategy::Sync) {
                                                    let mut tab = entry.lock();
                                                    tab.resize(r, c);
                                                }
                                            }
                                        }
                                        ClientMsg::Subscribe { rows: r, cols: c, .. } => {
                                            sub_rows = r;
                                            sub_cols = c;
                                            last_sent_screen = None; // Force full screen
                                        }
                                        ClientMsg::Input { tab_id: ref tid, ref data } => {
                                            if let Some(tab) = tabs.get(tid) {
                                                let mut tab = tab.lock();
                                                tab.ensure_running();
                                                tab.write(data);
                                            }
                                            skip_coalesce = true;
                                        }
                                        ClientMsg::Paste { tab_id: ref tid, ref data } => {
                                            if let Some(tab) = tabs.get(tid) {
                                                let mut tab = tab.lock();
                                                tab.ensure_running();
                                                tab.paste(data);
                                            }
                                            skip_coalesce = true;
                                        }
                                        _ => {}
                                    }
                                    continue;
                                }
                                Err(_) => break,
                            }
                        }
                    }

                    // Adaptive coalesce: skip for interactive keystrokes (echo path),
                    // only coalesce during rapid output bursts (e.g. cat large_file)
                    if !skip_coalesce {
                        let has_more = tokio::time::timeout(
                            tokio::time::Duration::from_micros(500),
                            notify.notified(),
                        )
                        .await
                        .is_ok();
                        if has_more {
                            // Rapid output — coalesce 1ms more to batch
                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                    }
                    skip_coalesce = false;

                    let Some(entry) = tabs.get(&tab_id) else {
                        break;
                    };
                    let content = {
                        let tab = entry.lock();
                        tab.get_screen(sub_rows, sub_cols)
                    };

                    // Try incremental diff against last sent screen
                    let bin = if let Some(ref prev) = last_sent_screen {
                        // Compare line by line
                        let mut changed: Vec<(u16, crate::terminal_provider::ScreenLine)> =
                            Vec::new();
                        let max_lines = content.lines.len().max(prev.lines.len());
                        for i in 0..max_lines {
                            let new_line = content.lines.get(i);
                            let old_line = prev.lines.get(i);
                            if new_line != old_line {
                                if let Some(line) = new_line {
                                    changed.push((i as u16, line.clone()));
                                }
                            }
                        }
                        let meta_changed = content.cursor != prev.cursor
                            || content.cursor_shape != prev.cursor_shape
                            || content.title != prev.title
                            || content.bell;

                        if changed.is_empty() && !meta_changed {
                            // Truly unchanged
                            continue;
                        }

                        // Use diff if fewer than half the lines changed (or metadata-only)
                        if changed.len() <= max_lines / 2 {
                            let diff = ServerMsg::ScreenDiff {
                                changed_lines: changed,
                                cursor: content.cursor,
                                cursor_shape: content.cursor_shape.clone(),
                                title: content.title.clone(),
                                bell: content.bell,
                            };
                            rmp_serde::to_vec(&diff).unwrap_or_default()
                        } else {
                            // Too many changes — send full screen
                            let response = ServerMsg::Screen {
                                tab_id: tab_id.clone(),
                                content: content.clone(),
                            };
                            rmp_serde::to_vec(&response).unwrap_or_default()
                        }
                    } else {
                        let response = ServerMsg::Screen {
                            tab_id: tab_id.clone(),
                            content: content.clone(),
                        };
                        rmp_serde::to_vec(&response).unwrap_or_default()
                    };

                    last_sent_screen = Some(content);
                    let len = (bin.len() as u32).to_le_bytes();
                    // Reuse pre-allocated buffer to avoid per-push allocation
                    push_frame_buf.clear();
                    push_frame_buf.push(0x00);
                    push_frame_buf.extend_from_slice(&len);
                    push_frame_buf.extend_from_slice(&bin);
                    if writer.write_all(&push_frame_buf).await.is_err() {
                        break;
                    }
                    if writer.flush().await.is_err() {
                        break;
                    }
                }
                tracing::debug!("Client #{} push loop ended for tab {}", client_id, tab_id);
                break; // Exit handle_client after push loop ends
            }
        };

        // Use MessagePack for Screen responses (hot path), JSON for everything else.
        if matches!(response, ServerMsg::Screen { .. }) {
            let bin = rmp_serde::to_vec(&response).unwrap_or_default();
            let len = (bin.len() as u32).to_le_bytes();
            writer.write_all(&[0x00]).await?; // magic byte: binary frame
            writer.write_all(&len).await?;
            writer.write_all(&bin).await?;
        } else {
            let json = serde_json::to_vec(&response)?;
            writer.write_all(&json).await?;
            writer.write_all(b"\n").await?;
        }
        writer.flush().await?;
    }

    Ok(())
}
