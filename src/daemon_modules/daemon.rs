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
use crate::daemon_modules::signals::SignalHandler;
use crate::config::Config;

pub struct Daemon {
    tabs: Arc<DashMap<String, Arc<Mutex<PtyProvider>>>>,
    signals: SignalHandler,
    config: Arc<Mutex<Config>>,
    client_count: Arc<AtomicUsize>,
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
                            
                            tokio::spawn(async move {
                                if let Err(e) = handle_client(stream, tabs, config, client_id).await {
                                    tracing::error!("Client #{} error: {}", client_id, e);
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

pub async fn handle_client(
    stream: UnixStream,
    tabs: Arc<DashMap<String, Arc<Mutex<PtyProvider>>>>,
    config: Arc<Mutex<Config>>,
    client_id: usize,
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
                    tracing::info!("Tab {} already exists, skipping spawn", tab_id);
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
            
            ClientMsg::Resize { tab_id, rows, cols } => {
                if let Some(tab) = tabs.get(&tab_id) {
                    (*tab.lock()).resize(rows, cols);
                }
                continue;
            }
            
            ClientMsg::GetScreen { tab_id, rows, cols } => {
                if let Some(tab) = tabs.get(&tab_id) {
                    let content = (*tab.lock()).get_screen(rows, cols);
                    ServerMsg::Screen { tab_id, content }
                } else {
                    tracing::warn!("Tab not found: {}", tab_id);
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
        };

        let json = serde_json::to_vec(&response)?;
        writer.write_all(&json).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }

    Ok(())
}
