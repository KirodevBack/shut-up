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
    pub clear_cache: &'static str,
    pub clear_cache_warning: &'static str,
    pub clear_cache_done: &'static str,
    pub clear_cache_failed: &'static str,
    #[cfg(not(windows))]
    pub continue_button: &'static str,
    #[cfg(not(windows))]
    pub cancel_button: &'static str,
}

impl Default for Labels {
    fn default() -> Self {
        Self {
            submenu: "Shut it up",
            toggle_all: "All",
            quit: "Quit",
            clear_cache: "Clean cached RAM",
            clear_cache_warning: "This empties the cached RAM of the whole computer to free it.\n\n\
Some stuff can stutter for a few seconds right after, so it's not a good idea to do it while gaming.\n\n\
Admin rights will be asked. Continue?",
            clear_cache_done: "Cached RAM cleaned.",
            clear_cache_failed: "Couldn't clean the cached RAM.",
            #[cfg(not(windows))]
            continue_button: "Continue",
            #[cfg(not(windows))]
            cancel_button: "Cancel",
        }
    }
}

impl Labels {
    pub fn not_running_message(&self, names: &[&str]) -> Option<String> {
        let (last, rest) = names.split_last()?;
        let list = if rest.is_empty() {
            last.to_string()
        } else {
            format!("{} and {last}", rest.join(", "))
        };
        let verb = if rest.is_empty() {
            "isn't running or isn't installed"
        } else {
            "aren't running or aren't installed"
        };
        Some(format!("{list} {verb}.\n\nIt'll get shut up as soon as you open it."))
    }
}
