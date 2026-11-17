use crate::config::{Config, Target};
use crate::platform::process;
use crate::state;
use std::sync::atomic::{AtomicU32, Ordering};

pub struct App {
    config: Config,
    enabled: AtomicU32,
}

impl App {
    pub fn init(config: Config) -> &'static App {
        let enabled = state::load(&config.targets);
        Box::leak(Box::new(App {
            config,
            enabled: AtomicU32::new(enabled),
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

    pub fn toggle(&self, index: usize) {
        self.set_mask(self.mask() ^ 1 << index);
    }

    pub fn toggle_all(&self) {
        self.set_mask(if self.all_enabled() { 0 } else { self.all_mask() });
    }

    pub fn enforce(&self) {
        self.apply(self.mask(), true);
    }

    pub fn shutdown(&self) {
        if self.config.restore_on_exit {
            self.apply(self.mask(), false);
        }
    }

    fn set_mask(&self, new: u32) {
        let old = self.enabled.swap(new, Ordering::Relaxed);
        let changed = old ^ new;
        if changed == 0 {
            return;
        }
        self.apply(changed & new, true);
        self.apply(changed & !new, false);
        state::save(self.targets(), new);
    }

    fn apply(&self, mask: u32, efficient: bool) {
        if mask == 0 {
            return;
        }
        let trim = efficient && self.config.trim_memory;
        process::for_each(|pid, name| {
            let hit = self
                .targets()
                .iter()
                .enumerate()
                .any(|(i, t)| mask >> i & 1 == 1 && t.matches(name));
            if hit {
                process::set_efficiency(pid, efficient, trim);
            }
        });
    }
}
