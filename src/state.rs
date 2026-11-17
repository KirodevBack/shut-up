use crate::config::Target;
use crate::platform;
use std::fs;
use std::path::PathBuf;

fn path() -> Option<PathBuf> {
    platform::config_dir().map(|dir| dir.join("enabled"))
}

pub fn load(targets: &[Target]) -> u32 {
    let Some(text) = path().and_then(|p| fs::read_to_string(p).ok()) else {
        return 0;
    };
    text.lines()
        .filter_map(|line| targets.iter().position(|t| t.name == line.trim()))
        .fold(0, |mask, index| mask | 1 << index)
}

pub fn save(targets: &[Target], mask: u32) {
    let Some(path) = path() else { return };
    let text: String = targets
        .iter()
        .enumerate()
        .filter(|(i, _)| mask >> i & 1 == 1)
        .map(|(_, t)| format!("{}\n", t.name))
        .collect();
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(path, text);
}
