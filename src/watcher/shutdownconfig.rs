use std::sync::{atomic, Arc};

#[derive(Clone)]
pub struct ShutdownConfig {
    flag: Arc<atomic::AtomicBool>,
    result: Arc<atomic::AtomicBool>, // true if stream finished
}

impl ShutdownConfig {
    pub fn new(flag: Arc<atomic::AtomicBool>, result: Arc<atomic::AtomicBool>) -> Self {
        Self { flag, result }
    }

    pub fn shutdown(&self) {
        self.flag.store(true, atomic::Ordering::SeqCst);
    }

    pub fn is_shutdown(&self) -> bool {
        self.flag.load(atomic::Ordering::SeqCst)
    }

    pub fn is_finished(&self) -> bool {
        self.result.load(atomic::Ordering::SeqCst)
    }

    pub fn finish(&self) {
        self.result.store(true, atomic::Ordering::SeqCst);
    }
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            flag: Arc::new(atomic::AtomicBool::new(false)),
            result: Arc::new(atomic::AtomicBool::new(false)),
        }
    }
}
