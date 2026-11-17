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
        let helper_suffix = " helper";
        self.processes.iter().any(|name| {
            let same = process.eq_ignore_ascii_case(name);
            let helper_end = name.len() + helper_suffix.len();
            let helper = process
                .get(..name.len())
                .is_some_and(|start| start.eq_ignore_ascii_case(name))
                && process
                    .get(name.len()..helper_end)
                    .is_some_and(|rest| rest.eq_ignore_ascii_case(helper_suffix));
            same || helper
        })
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
refresh_seconds = {refresh}
trim_memory = {trim}
restore_on_exit = {restore}

[Discord]
processes = discord, discordcanary, discordptb, vesktop

[Spotify]
processes = spotify

[LibreWolf]
processes = librewolf

[Firefox]
processes = firefox

[Chrome]
processes = chrome, google chrome

[Edge]
processes = msedge, microsoft edge

[Brave]
processes = brave, brave browser

[Opera]
processes = opera

[Vivaldi]
processes = vivaldi

[Claude]
processes = claude

[Visual Studio Code]
processes = code
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
        let defaults = Self::default();
        match fs::read_to_string(&path) {
            Ok(text) => {
                let text = text.trim_start_matches('\u{feff}').to_owned();
                let upgraded = Self::upgrade(&text, &defaults);
                if upgraded != text {
                    let _ = fs::write(&path, &upgraded);
                }
                remember_defaults(&defaults.targets);
                Self::from_user_text(&upgraded)
            }
            Err(_) => {
                if let Some(dir) = path.parent() {
                    let _ = fs::create_dir_all(dir);
                }
                let _ = fs::write(&path, Self::default_text());
                remember_defaults(&defaults.targets);
                defaults
            }
        }
    }

    fn upgrade(text: &str, defaults: &Config) -> String {
        let default_text = Self::default_text();
        let user_keys: Vec<&str> = global_lines(text).filter_map(key_of).collect();
        let missing_keys: Vec<&str> = global_lines(&default_text)
            .filter(|line| key_of(line).is_some_and(|key| !user_keys.contains(&key)))
            .collect();

        let mut upgraded = text.to_owned();
        if !missing_keys.is_empty() {
            let block: String = missing_keys.iter().map(|line| format!("{line}\n")).collect();
            let first_section = upgraded
                .match_indices('[')
                .map(|(at, _)| at)
                .find(|&at| at == 0 || upgraded[..at].ends_with('\n'));
            upgraded = match first_section {
                Some(at) => {
                    let head = upgraded[..at].trim_end();
                    let gap = if head.is_empty() { "" } else { "\n" };
                    format!("{head}{gap}{block}\n{}", &upgraded[at..])
                }
                None => format!("{block}{upgraded}"),
            };
        }

        let sections = section_names(text);
        let seen = seen_defaults();
        let is_known = |name: &str| {
            sections.iter().chain(&seen).any(|known| known.eq_ignore_ascii_case(name))
        };
        for target in defaults.targets.iter().filter(|t| !is_known(&t.name)) {
            if !upgraded.is_empty() && !upgraded.ends_with('\n') {
                upgraded.push('\n');
            }
            upgraded.push_str(&format!(
                "\n[{}]\nprocesses = {}\n",
                target.name,
                target.processes.join(", ")
            ));
        }
        upgraded
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

fn global_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines().take_while(|line| !line.trim_start().starts_with('['))
}

fn key_of(line: &str) -> Option<&str> {
    let line = line.trim();
    if line.starts_with('#') || line.starts_with(';') {
        return None;
    }
    line.split_once('=').map(|(key, _)| key.trim())
}

fn section_names(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix('[')?.strip_suffix(']'))
        .map(|name| name.trim().to_owned())
        .collect()
}

fn seen_path() -> Option<PathBuf> {
    platform::config_dir().map(|dir| dir.join("defaults-seen"))
}

fn seen_defaults() -> Vec<String> {
    seen_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|text| text.lines().map(|line| line.trim().to_owned()).collect())
        .unwrap_or_default()
}

fn remember_defaults(targets: &[Target]) {
    let Some(path) = seen_path() else { return };
    let mut names = seen_defaults();
    for target in targets {
        if !names.iter().any(|name| name.eq_ignore_ascii_case(&target.name)) {
            names.push(target.name.clone());
        }
    }
    let text: String = names.iter().map(|name| format!("{name}\n")).collect();
    if fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
        let _ = fs::write(path, text);
    }
}

fn parse_bool(value: &str, fallback: bool) -> bool {
    match value.to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => true,
        "false" | "no" | "off" | "0" => false,
        _ => fallback,
    }
}
