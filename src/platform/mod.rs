#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::{config_dir, process, run};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::{config_dir, process, run};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use self::macos::{config_dir, process, run};

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
    pub not_running: &'static str,
    pub clear_cache: &'static str,
    pub clear_cache_warning: &'static str,
    pub clear_cache_done: &'static str,
    pub clear_cache_failed: &'static str,
}

impl Default for Labels {
    fn default() -> Self {
        Self {
            submenu: "Shut it up",
            toggle_all: "Tout",
            quit: "Quitter",
            not_running: "pas lancé ou pas installé",
            clear_cache: "Vider la RAM en cache",
            clear_cache_warning: "Ça vide la RAM en cache de tout l'ordi pour la libérer.

Certains trucs peuvent saccader quelques secondes juste après, c'est pas conseillé de le faire en jeu.

Les droits administrateur vont être demandés. Continuer ?",
            clear_cache_done: "RAM en cache vidée.",
            clear_cache_failed: "Impossible de vider la RAM en cache.",
        }
    }
}

impl Labels {
    pub fn target(&self, name: &str, running: bool) -> String {
        if running {
            name.to_owned()
        } else {
            format!("{name}   ⚠ {}", self.not_running)
        }
    }
}
