use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct SignalHandler {
    shutdown: Arc<AtomicBool>,
    reload: Arc<AtomicBool>,
}

impl SignalHandler {
    pub fn new() -> Self {
        let shutdown = Arc::new(AtomicBool::new(false));
        let reload = Arc::new(AtomicBool::new(false));

        #[cfg(unix)]
        {
            let shutdown_clone = shutdown.clone();
            let reload_clone = reload.clone();
            std::thread::spawn(move || unsafe {
                let mut sigset: libc::sigset_t = std::mem::zeroed();
                libc::sigemptyset(&mut sigset);
                libc::sigaddset(&mut sigset, libc::SIGTERM);
                libc::sigaddset(&mut sigset, libc::SIGINT);
                libc::sigaddset(&mut sigset, libc::SIGHUP);
                libc::pthread_sigmask(libc::SIG_BLOCK, &sigset, std::ptr::null_mut());

                loop {
                    let mut sig: libc::c_int = 0;
                    if libc::sigwait(&sigset, &mut sig) == 0 {
                        match sig {
                            libc::SIGTERM | libc::SIGINT => {
                                shutdown_clone.store(true, Ordering::Relaxed);
                                break;
                            }
                            libc::SIGHUP => {
                                reload_clone.store(true, Ordering::Relaxed);
                            }
                            _ => {}
                        }
                    }
                }
            });
        }

        Self { shutdown, reload }
    }

    pub fn should_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }

    pub fn should_reload(&self) -> bool {
        self.reload.swap(false, Ordering::Relaxed)
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}
