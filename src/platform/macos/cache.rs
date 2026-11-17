use crate::platform::{app_name, Labels};
use std::process::Command;
use std::thread;

fn quoted(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

fn osascript(script: &str) -> bool {
    Command::new("osascript")
        .args(["-e", script])
        .status()
        .is_ok_and(|status| status.success())
}

pub fn clear(labels: &Labels) {
    let (done, failed) = (labels.clear_cache_done, labels.clear_cache_failed);
    thread::spawn(move || {
        let purged = osascript("do shell script \"purge\" with administrator privileges");
        let text = if purged { done } else { failed };
        osascript(&format!("display notification {} with title {}", quoted(text), quoted(app_name())));
    });
}
