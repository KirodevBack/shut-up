use crate::app::App;
use crate::platform;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Target {
    pub name: String,
    pub processes: Vec<String>,
}

impl Target {
    pub fn matches(&self, process: &str) -> bool {
        let process = process.strip_suffix(".exe").unwrap_or(process);
        self.processes.iter().any(|p| p.eq_ignore_ascii_case(process))
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub refresh_interval: Duration,
    pub trim_memory: bool,
    pub restore_on_exit: bool,
    pub targets: Vec<Target>,
}

impl Default for Config {
    fn default() -> Self {
        Self::parse(&Self::default_text())
    }
}

impl Config {
    fn base() -> Self {
        Self {
            refresh_interval: Duration::from_secs(30),
            trim_memory: true,
            restore_on_exit: true,
            targets: Vec::new(),
        }
    }

    fn default_text() -> String {
        let base = Self::base();
        format!(
            "\
# Configuration de shut-up
# Chaque [section] devient une entree du menu \"Shut it up\".
# processes = noms des executables, sans .exe, majuscules ignorees.
# refresh_seconds = intervalle de reapplication (minimum 5).

refresh_seconds = {refresh}
trim_memory = {trim}
restore_on_exit = {restore}

[Discord]
processes = discord, discordcanary, discordptb, vesktop

[Spotify]
processes = spotify
",
            refresh = base.refresh_interval.as_secs(),
            trim = base.trim_memory,
            restore = base.restore_on_exit,
        )
    }

    pub fn path() -> Option<PathBuf> {
        platform::config_dir().map(|dir| dir.join("config.ini"))
    }

    pub fn load_or_create() -> Self {
        let Some(path) = Self::path() else {
            return Self::default();
        };
        match fs::read_to_string(&path) {
            Ok(text) => Self::from_user_text(&text),
            Err(_) => {
                if let Some(dir) = path.parent() {
                    let _ = fs::create_dir_all(dir);
                }
                let _ = fs::write(&path, Self::default_text());
                Self::default()
            }
        }
    }

    fn from_user_text(text: &str) -> Self {
        let mut config = Self::parse(text);
        if config.targets.is_empty() {
            config.targets = Self::default().targets;
        }
        config
    }

    fn parse(text: &str) -> Self {
        let min_refresh = Duration::from_secs(5);
        let mut config = Self::base();

        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                config.targets.push(Target {
                    name: name.trim().to_owned(),
                    processes: Vec::new(),
                });
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());

            match (config.targets.last_mut(), key) {
                (Some(target), "processes") => {
                    target.processes.extend(
                        value
                            .split(',')
                            .map(str::trim)
                            .filter(|p| !p.is_empty())
                            .map(str::to_owned),
                    );
                }
                (None, "refresh_seconds") => {
                    if let Ok(secs) = value.parse::<u64>() {
                        config.refresh_interval = Duration::from_secs(secs).max(min_refresh);
                    }
                }
                (None, "trim_memory") => config.trim_memory = parse_bool(value, config.trim_memory),
                (None, "restore_on_exit") => {
                    config.restore_on_exit = parse_bool(value, config.restore_on_exit)
                }
                _ => {}
            }
        }

        config.targets.retain(|t| !t.processes.is_empty());
        config.targets.truncate(App::capacity());
        config
    }
}

fn parse_bool(value: &str, fallback: bool) -> bool {
    match value.to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => true,
        "false" | "no" | "off" | "0" => false,
        _ => fallback,
    }
}
