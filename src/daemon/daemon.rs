use anyhow::Result;
use dashmap::DashMap;
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use crate::protocol::{ClientMsg, ServerMsg};
use crate::providers::PtyProvider;
use crate::terminal_provider::TerminalProvider;
use crate::daemon::signals::SignalHandler;
use crate::config::Config;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const VERSION_URL: &str = "https://mato.sh/version.txt";

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
                                let remote = text.trim().to_string();
                                let update = if remote != CURRENT_VERSION { Some(remote) } else { None };
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
                            tracing::info!("Client #{} connecting (total: {})", client_id, client_id);
                            
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
                                tracing::info!("Client #{} disconnected (remaining: {})", client_id, remaining);
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

pub async fn handle_client(
    stream: UnixStream,
    tabs: Arc<DashMap<String, Arc<Mutex<PtyProvider>>>>,
    _config: Arc<Mutex<Config>>,
    client_id: usize,
    latest_version: Arc<Mutex<Option<String>>>,
) -> Result<()> {
    tracing::info!("Client #{} handler started", client_id);
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 { 
            tracing::info!("Client #{} disconnected", client_id);
            break; 
        }

        let msg: ClientMsg = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Failed to parse message: {} | Line: {}", e, line.trim());
                continue;
            }
        };
        
        let response = match msg {
            ClientMsg::Hello { .. } => ServerMsg::Welcome { version: "0.1".into() },
            
            ClientMsg::Spawn { tab_id, rows, cols } => {
                if tabs.contains_key(&tab_id) {
                    tracing::info!("Tab {} already exists", tab_id);
                    // Don't resize on reconnect - it would clear the screen
                    // Client will send explicit Resize if needed
                    ServerMsg::Welcome { version: "already exists".into() }
                } else {
                    tracing::info!("Spawning new tab {} ({}x{})", tab_id, rows, cols);
                    let mut provider = PtyProvider::new();
                    provider.spawn(rows, cols);
                    tabs.insert(tab_id.clone(), Arc::new(Mutex::new(provider)));
                    ServerMsg::Welcome { version: "spawned".into() }
                }
            }
            
            ClientMsg::Input { tab_id, data } => {
                if let Some(tab) = tabs.get(&tab_id) {
                    (*tab.lock()).write(&data);
                }
                continue;
            }

            ClientMsg::Paste { tab_id, data } => {
                if let Some(tab) = tabs.get(&tab_id) {
                    (*tab.lock()).paste(&data);
                }
                continue;
            }

            ClientMsg::GetInputModes { tab_id } => {
                if let Some(tab) = tabs.get(&tab_id) {
                    let tab = tab.lock();
                    ServerMsg::InputModes {
                        mouse: tab.mouse_mode_enabled(),
                        bracketed_paste: tab.bracketed_paste_enabled(),
                    }
                } else {
                    ServerMsg::Error { message: "tab not found".into() }
                }
            }
            
            ClientMsg::Resize { tab_id, rows, cols } => {
                // DON'T resize the PTY! This would clear the screen.
                // The PTY should keep running at its original size.
                // Only the client's display needs to adapt to window size.
                tracing::debug!("Ignoring resize request for tab {} ({}x{}) - PTY size is fixed", tab_id, rows, cols);
                continue;
            }
            
            ClientMsg::GetScreen { tab_id, rows, cols } => {
                if let Some(tab) = tabs.get(&tab_id) {
                    let content = (*tab.lock()).get_screen(rows, cols);
                    ServerMsg::Screen { tab_id, content }
                } else {
                    tracing::debug!("Tab not found: {}", tab_id);
                    ServerMsg::Error { message: "tab not found".into() }
                }
            }

            ClientMsg::GetIdleStatus => {
                let now = Instant::now();
                let idle: Vec<(String, u64)> = tabs.iter().map(|entry| {
                    let secs = now.duration_since(*entry.value().lock().last_output.lock().unwrap()).as_secs();
                    (entry.key().clone(), secs)
                }).collect();
                ServerMsg::IdleStatus { tabs: idle }
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
                    entry.lock().scroll(delta);
                }
                continue;
            }
        };

        let json = serde_json::to_vec(&response)?;
        writer.write_all(&json).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }

    Ok(())
}
