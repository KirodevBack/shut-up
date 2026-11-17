#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::{config_dir, process, run};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::{config_dir, process, run};

pub fn app_id() -> &'static str {
    env!("CARGO_PKG_NAME")
}

pub fn app_name() -> &'static str {
    "Shut up"
}

pub struct Labels {
    pub submenu: &'static str,
    pub toggle_all: &'static str,
    pub quit: &'static str,
}

impl Default for Labels {
    fn default() -> Self {
        Self {
            submenu: "Shut it up",
            toggle_all: "Tout",
            quit: "Quitter",
        }
    }
}
