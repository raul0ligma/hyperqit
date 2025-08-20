use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};

pub struct NonceManager {
    counter: AtomicU64,
}

impl NonceManager {
    pub fn new() -> Self {
        NonceManager {
            counter: AtomicU64::new(0),
        }
    }
    pub fn get_next_nonce(&self) -> u64 {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let counter = self.counter.fetch_add(1, Ordering::Relaxed);
        timestamp.saturating_add(counter % 1000)
    }
}
