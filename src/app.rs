use crate::config::{Config, Target};
use crate::platform::process;
use crate::state;
use std::sync::atomic::{AtomicU32, Ordering};

pub struct App {
    config: Config,
    enabled: AtomicU32,
    running: AtomicU32,
}

impl App {
    pub fn init(config: Config) -> &'static App {
        let enabled = state::load(&config.targets);
        Box::leak(Box::new(App {
            config,
            enabled: AtomicU32::new(enabled),
            running: AtomicU32::new(0),
        }))
    }

    pub fn capacity() -> usize {
        u32::BITS as usize
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn targets(&self) -> &[Target] {
        &self.config.targets
    }

    fn all_mask(&self) -> u32 {
        let count = self.targets().len() as u32;
        u32::MAX.checked_shr(u32::BITS - count).unwrap_or(0)
    }

    fn mask(&self) -> u32 {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn is_enabled(&self, index: usize) -> bool {
        self.mask() >> index & 1 == 1
    }

    pub fn all_enabled(&self) -> bool {
        self.mask() == self.all_mask()
    }

    pub fn toggle(&self, index: usize) -> Vec<&str> {
        self.set_mask(self.mask() ^ 1 << index)
    }

    pub fn toggle_all(&self) -> Vec<&str> {
        self.set_mask(if self.all_enabled() { 0 } else { self.all_mask() })
    }

    pub fn enforce(&self) {
        let running = self.scan(self.mask(), true);
        self.running.store(running, Ordering::Relaxed);
    }

    pub fn shutdown(&self) {
        if self.config.restore_on_exit {
            self.apply(self.mask(), false);
        }
    }

    fn set_mask(&self, new: u32) -> Vec<&str> {
        let old = self.enabled.swap(new, Ordering::Relaxed);
        let changed = old ^ new;
        if changed == 0 {
            return Vec::new();
        }
        let turned_on = changed & new;
        self.apply(turned_on, true);
        self.apply(changed & !new, false);
        state::save(self.targets(), new);
        self.not_running(turned_on)
    }

    fn not_running(&self, mask: u32) -> Vec<&str> {
        let running = self.running.load(Ordering::Relaxed);
        self.targets()
            .iter()
            .enumerate()
            .filter(|(i, _)| mask >> i & 1 == 1 && running >> i & 1 == 0)
            .map(|(_, t)| t.name.as_str())
            .collect()
    }

    fn apply(&self, mask: u32, efficient: bool) {
        if mask != 0 {
            let running = self.scan(mask, efficient);
            self.running.store(running, Ordering::Relaxed);
        }
    }

    fn scan(&self, mask: u32, efficient: bool) -> u32 {
        let trim = efficient && self.config.trim_memory;
        let mut running = 0;
        process::for_each(|pid, name| {
            let hits = self
                .targets()
                .iter()
                .enumerate()
                .filter(|(_, t)| t.matches(name))
                .fold(0, |bits, (i, _)| bits | 1 << i);
            running |= hits;
            if hits & mask != 0 {
                process::set_efficiency(pid, efficient, trim);
            }
        });
        running
    }
}
